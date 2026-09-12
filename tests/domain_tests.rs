use bookfin::domain::errors::DomainError;
use bookfin::domain::models::{LanguageTag, ReactionType};
use std::str::FromStr;

#[test]
fn test_language_tag_rejects_empty() {
    let empty_res = LanguageTag::parse("");
    assert_eq!(empty_res, Err(DomainError::EmptyLanguageTag));

    let whitespace_res = LanguageTag::parse("   ");
    assert_eq!(whitespace_res, Err(DomainError::EmptyLanguageTag));
}

#[test]
fn test_language_tag_rejects_invalid_chars() {
    let invalid_res = LanguageTag::parse("en@invalid!");
    assert!(matches!(
        invalid_res,
        Err(DomainError::InvalidLanguageTag(_))
    ));
}

#[test]
fn test_language_tag_valid_values() {
    let tag = LanguageTag::parse("en-US").expect("en-US doit être valide");
    assert_eq!(tag.as_str(), "en-us");

    let tag_fr = LanguageTag::parse("FR").expect("FR doit être valide");
    assert_eq!(tag_fr.as_str(), "fr");
}

#[test]
fn test_reaction_type_parsing() {
    assert_eq!(ReactionType::from_str("like").unwrap(), ReactionType::Like);
    assert_eq!(ReactionType::from_str("SKIP").unwrap(), ReactionType::Skip);
    assert_eq!(ReactionType::from_str("save").unwrap(), ReactionType::Save);
    assert_eq!(
        ReactionType::from_str("reveal").unwrap(),
        ReactionType::Reveal
    );
    assert!(ReactionType::from_str("unknown").is_err());
}

#[test]
fn test_server_timing_validation() {
    use bookfin::domain::models::{validate_server_timing, ReadingValidationConfig};
    use chrono::{Duration, Utc};

    let config = ReadingValidationConfig::default();
    let now = Utc::now();

    // 2000 ms elapsed < 3500 ms -> rejected
    let served_at_fast = now - Duration::milliseconds(2000);
    let result_fast = validate_server_timing(served_at_fast, now);
    assert!(matches!(
        result_fast,
        Err(DomainError::ServerTimeElapsedTooShort { elapsed_ms, min_required_ms })
        if elapsed_ms < 3500 && min_required_ms == config.min_server_elapsed_ms
    ));

    // 4000 ms elapsed >= 3500 ms -> accepted
    let served_at_ok = now - Duration::milliseconds(4000);
    assert!(validate_server_timing(served_at_ok, now).is_ok());
}

#[test]
fn test_reading_metrics_guardrails() {
    use bookfin::domain::models::validate_reading_metrics;

    // Reading too fast (< 4000 ms)
    let fast_res = validate_reading_metrics(2500, 0.90);
    assert!(matches!(fast_res, Err(DomainError::ReadingTooFast { .. })));

    // Insufficient scroll (< 0.75)
    let low_scroll = validate_reading_metrics(5000, 0.60);
    assert!(matches!(
        low_scroll,
        Err(DomainError::InsufficientScroll { .. })
    ));

    // Both valid
    let ok_res = validate_reading_metrics(4500, 0.85);
    assert!(ok_res.is_ok());
}

#[test]
fn test_page_content_hash_deterministic() {
    use bookfin::domain::models::Page;

    let text1 = "Il pleuvait doucement sur la ville endormie.";
    let text1_whitespace = "Il   pleuvait doucement   sur la ville endormie.  ";
    let text2 = "Le soleil brillait sur la mer Méditerranée.";

    let hash1 = Page::compute_hash(text1);
    let hash1_ws = Page::compute_hash(text1_whitespace);
    let hash2 = Page::compute_hash(text2);

    assert_eq!(
        hash1, hash1_ws,
        "Whitespace differences should normalize to identical SHA-256"
    );
    assert_ne!(
        hash1, hash2,
        "Different contents must produce different hashes"
    );
    assert_eq!(hash1.len(), 64, "SHA-256 hex string must be 64 characters");
}

#[test]
fn test_reading_stats_model() {
    use bookfin::domain::models::ReadingStats;
    use uuid::Uuid;

    let stats = ReadingStats {
        user_id: Uuid::new_v4(),
        pages_served: 150,
        pages_reacted: 120,
        likes: 70,
        dislikes: 30,
        skips: 15,
        saves: 5,
    };

    assert_eq!(stats.pages_served, 150);
    assert_eq!(stats.pages_reacted, 120);
    assert_eq!(
        stats.likes + stats.dislikes + stats.skips + stats.saves,
        120
    );
}

#[test]
fn test_navigation_action_parsing_and_display() {
    use bookfin::domain::models::NavigationAction;

    assert_eq!(
        NavigationAction::from_str("continue_book").unwrap(),
        NavigationAction::ContinueBook
    );
    assert_eq!(
        NavigationAction::from_str("CONTINUE_BOOK").unwrap(),
        NavigationAction::ContinueBook
    );
    assert_eq!(
        NavigationAction::from_str("random_page").unwrap(),
        NavigationAction::RandomPage
    );
    assert_eq!(
        NavigationAction::from_str("RANDOM_PAGE").unwrap(),
        NavigationAction::RandomPage
    );
    assert!(NavigationAction::from_str("unknown_action").is_err());

    assert_eq!(NavigationAction::ContinueBook.as_str(), "continue_book");
    assert_eq!(NavigationAction::RandomPage.as_str(), "random_page");
}

#[test]
fn test_flexible_scroll_validation_scenarios() {
    use bookfin::domain::models::{validate_reading_metrics_full, ReadingValidationConfig};

    let config = ReadingValidationConfig::default();

    // Scenario 1: Short page that does not overflow the screen (content_overflows = false)
    // Even with 0% scroll, as long as reading time >= 4000ms, it is VALID.
    let short_page_res = validate_reading_metrics_full(4500, 0.0, false, false, &config);
    assert!(
        short_page_res.is_ok(),
        "Short non-overflowing page should not require scrolling"
    );

    // Scenario 2: Content overflows, but bottom was reached (bottom_reached = true)
    // E.g. fast jump or screen size allowed reaching the bottom with 50% scroll ratio
    let bottom_reached_res = validate_reading_metrics_full(5000, 0.50, true, true, &config);
    assert!(
        bottom_reached_res.is_ok(),
        "Reaching bottom should satisfy scroll requirement even if ratio is low"
    );

    // Scenario 3: Content overflows, bottom NOT reached, scroll ratio insufficient (< 0.75)
    let incomplete_scroll_res = validate_reading_metrics_full(5000, 0.60, false, true, &config);
    assert!(
        matches!(
            incomplete_scroll_res,
            Err(DomainError::InsufficientScroll { .. })
        ),
        "Overflowing content without reaching bottom or 75% scroll must be rejected"
    );

    // Scenario 4: Content overflows, bottom NOT reached, but scroll ratio >= 0.75
    let sufficient_scroll_res = validate_reading_metrics_full(5000, 0.80, false, true, &config);
    assert!(
        sufficient_scroll_res.is_ok(),
        "75%+ scroll should be accepted even if exact bottom was not touched"
    );

    // Scenario 5: Reading time too short (< 4000ms) - rejected regardless of scroll or overflow
    let too_fast_res = validate_reading_metrics_full(2000, 1.0, true, false, &config);
    assert!(
        matches!(too_fast_res, Err(DomainError::ReadingTooFast { .. })),
        "Reading too fast must always be rejected"
    );
}

#[test]
fn test_page_sequence_vs_source_page_number() {
    use bookfin::domain::models::Page;
    use uuid::Uuid;

    let page_roman = Page::new(
        Uuid::new_v4(),
        Uuid::new_v4(),
        1,
        Some("XII".to_string()),
        "Préface de l'auteur...",
        "fr".to_string(),
        50,
        0.1,
    );
    assert_eq!(page_roman.page_sequence_number(), 1);
    assert_eq!(page_roman.source_page_number.as_deref(), Some("XII"));

    let page_unpaginated = Page::new(
        Uuid::new_v4(),
        Uuid::new_v4(),
        42,
        None,
        "Chapitre suivant...",
        "fr".to_string(),
        80,
        0.5,
    );
    assert_eq!(page_unpaginated.page_sequence_number(), 42);
    assert_eq!(page_unpaginated.source_page_number, None);
}
