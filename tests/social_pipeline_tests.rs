use chrono::Utc;
use uuid::Uuid;

use bookfin::domain::errors::DomainError;
use bookfin::domain::models::{
    validate_reading_metrics, ConnectionProposal, ConnectionStatus, ReactionType, UserAffinity,
    AFFINITY_HALF_CONFIDENCE_K, MIN_AFFINITY_CONNECT_THRESHOLD, MIN_READING_TIME_MS,
    MIN_SCROLL_DEPTH,
};

#[test]
fn test_reading_threshold_too_fast_rejected() {
    // 3999 ms est inférieur au seuil minimal de 4000 ms
    let result = validate_reading_metrics(3999, 0.85);
    assert!(result.is_err());
    match result.unwrap_err() {
        DomainError::ReadingTooFast {
            reading_time_ms,
            min_required_ms,
        } => {
            assert_eq!(reading_time_ms, 3999);
            assert_eq!(min_required_ms, MIN_READING_TIME_MS);
        }
        other => panic!("Erreur inattendue: {:?}", other),
    }
}

#[test]
fn test_reading_threshold_insufficient_scroll_rejected() {
    // 0.70 est inférieur au seuil minimal de 0.75 (75%)
    let result = validate_reading_metrics(5000, 0.70);
    assert!(result.is_err());
    match result.unwrap_err() {
        DomainError::InsufficientScroll { .. } => {}
        other => panic!("Erreur inattendue: {:?}", other),
    }
}

#[test]
fn test_reading_threshold_valid() {
    let result = validate_reading_metrics(4500, 0.80);
    assert!(result.is_ok());

    let result_exact = validate_reading_metrics(MIN_READING_TIME_MS, MIN_SCROLL_DEPTH);
    assert!(result_exact.is_ok());
}

#[test]
fn test_reaction_type_parsing_social() {
    let like: ReactionType = "like".parse().expect("Parse like");
    let dislike: ReactionType = "dislike".parse().expect("Parse dislike");
    assert_eq!(like, ReactionType::Like);
    assert_eq!(dislike, ReactionType::Dislike);

    let invalid = "invalid_reaction".parse::<ReactionType>();
    assert!(invalid.is_err());
}

#[test]
fn test_affinity_calculation_exact() {
    let target_user = Uuid::new_v4();
    let common_likes = 15;
    let common_dislikes = 5;
    let disagreements = 10;

    let aff = UserAffinity::compute(target_user, common_likes, common_dislikes, disagreements);

    assert_eq!(aff.target_user_id, target_user);
    assert_eq!(aff.common_likes, 15);
    assert_eq!(aff.common_dislikes, 5);
    assert_eq!(aff.disagreements, 10);
    assert_eq!(aff.comparable_volume, 30);

    // Accord brut : (15 + 5) / 30 = 20 / 30 = 2/3 ≈ 0.6667
    let expected_raw = 20.0 / 30.0;
    assert!((aff.raw_agreement - expected_raw).abs() < 1e-6);

    // Confiance : 30 / (30 + 20) = 30 / 50 = 0.6
    let expected_confidence = 30.0 / (30.0 + AFFINITY_HALF_CONFIDENCE_K);
    assert!((aff.confidence - expected_confidence).abs() < 1e-6);

    // Score final : (20/30) * (30/50) = 20 / 50 = 0.40
    let expected_score = 20.0 / (30.0 + AFFINITY_HALF_CONFIDENCE_K);
    assert!((aff.affinity_score - expected_score).abs() < 1e-6);
    assert!(
        !aff.can_connect,
        "Score 0.40 < 0.50 : pas de proposition de connexion"
    );
}

#[test]
fn test_affinity_low_confidence_with_low_overlap() {
    let target_user = Uuid::new_v4();
    // 1 like commun sur 1 seule lecture comparable (100% d'accord brut)
    let aff = UserAffinity::compute(target_user, 1, 0, 0);

    assert_eq!(aff.comparable_volume, 1);
    assert!(
        (aff.raw_agreement - 1.0).abs() < 1e-6,
        "Accord brut maximal à 1.0"
    );

    // Confiance = 1 / (1 + 20) ≈ 0.0476
    assert!(
        aff.confidence < 0.05,
        "Confiance très basse avec 1 lecture en commun"
    );
    // Score final amorti = 1 / 21 ≈ 0.0476
    assert!(
        aff.affinity_score < 0.05,
        "Le score reste faible malgré 100% d'accord"
    );
    assert!(
        !aff.can_connect,
        "Impossible de proposer une connexion sur 1 seul extrait"
    );
}

#[test]
fn test_affinity_high_confidence_with_high_overlap() {
    let target_user = Uuid::new_v4();
    // 80 likes communs, 15 dislikes communs, 5 désaccords = 100 lectures comparables
    let aff = UserAffinity::compute(target_user, 80, 15, 5);

    assert_eq!(aff.comparable_volume, 100);
    // Accord brut : 95 / 100 = 0.95
    assert!((aff.raw_agreement - 0.95).abs() < 1e-6);

    // Confiance : 100 / (100 + 20) = 100 / 120 ≈ 0.8333
    assert!(
        aff.confidence > 0.80,
        "Haute confiance sur 100 lectures comparables"
    );

    // Score final amorti : 95 / 120 ≈ 0.7917
    assert!(
        aff.affinity_score > 0.75,
        "Score élevé garanti par la masse de signal"
    );
    assert!(aff.affinity_score >= MIN_AFFINITY_CONNECT_THRESHOLD);
    assert!(
        aff.can_connect,
        "Connexion autorisée car affinité forte et fiable"
    );
}

#[test]
fn test_cannot_open_discussion_when_proposal_pending() {
    let proposal = ConnectionProposal {
        id: Uuid::new_v4(),
        requester_id: Uuid::new_v4(),
        recipient_id: Uuid::new_v4(),
        status: ConnectionStatus::Pending,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    assert_eq!(proposal.status, ConnectionStatus::Pending);
    assert!(
        !proposal.can_start_discussion(),
        "Une proposition pending ne doit pas permettre d'ouvrir une discussion"
    );
}

#[test]
fn test_cannot_open_discussion_when_proposal_rejected() {
    let proposal = ConnectionProposal {
        id: Uuid::new_v4(),
        requester_id: Uuid::new_v4(),
        recipient_id: Uuid::new_v4(),
        status: ConnectionStatus::Rejected,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    assert_eq!(proposal.status, ConnectionStatus::Rejected);
    assert!(
        !proposal.can_start_discussion(),
        "Une proposition rejetée ne doit jamais permettre d'ouvrir une discussion"
    );
}

#[test]
fn test_can_open_discussion_only_when_mutually_accepted() {
    let proposal = ConnectionProposal {
        id: Uuid::new_v4(),
        requester_id: Uuid::new_v4(),
        recipient_id: Uuid::new_v4(),
        status: ConnectionStatus::Accepted,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    assert_eq!(proposal.status, ConnectionStatus::Accepted);
    assert!(
        proposal.can_start_discussion(),
        "La discussion est ouverte uniquement si les deux utilisateurs ont mutuellement accepté"
    );
}
