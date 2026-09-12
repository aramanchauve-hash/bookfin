use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum DomainError {
    #[error("La chaîne du tag de langue ne peut pas être vide.")]
    EmptyLanguageTag,

    #[error("Le tag de langue '{0}' contient des caractères invalides.")]
    InvalidLanguageTag(String),

    #[error("Le type de réaction '{0}' n'est pas reconnu.")]
    InvalidReactionType(String),

    #[error("Entité non trouvée : {0}")]
    NotFound(String),

    #[error(
        "Lecture manifestement trop rapide ({reading_time_ms}ms < {min_required_ms}ms attendus)"
    )]
    ReadingTooFast {
        reading_time_ms: i32,
        min_required_ms: i32,
    },

    #[error("Progression de défilement insuffisante ({scroll_depth} < {min_required} attendu)")]
    InsufficientScroll {
        scroll_depth: String,
        min_required: String,
    },

    #[error("Page non trouvée : {0}")]
    PageNotFound(String),

    #[error("Impression non trouvée : {0}")]
    ImpressionNotFound(String),

    #[error("Temps serveur écoulé insuffisant depuis l'envoi de la page ({elapsed_ms}ms < {min_required_ms}ms attendus)")]
    ServerTimeElapsedTooShort {
        elapsed_ms: i64,
        min_required_ms: i64,
    },

    #[error("Réaction dupliquée pour cet événement")]
    DuplicateReaction,

    #[error("Signal insuffisant pour calculer l'affinité ({overlap} pages comparables < {min_required} requises)")]
    NotEnoughAffinityData { overlap: i32, min_required: i32 },

    #[error("Impossible de demander une connexion à soi-même")]
    SelfConnectionNotAllowed,

    #[error("Statut de connexion invalide: {0}")]
    InvalidConnectionStatus(String),

    #[error("Proposition de connexion non trouvée")]
    ConnectionProposalNotFound,

    #[error("Discussion non autorisée: les deux utilisateurs doivent d'abord accepter la mise en relation")]
    DiscussionNotAllowed,

    #[error("Fin de l'édition atteinte (édition {edition_id}, dernière page {last_page_number})")]
    EndOfEdition {
        edition_id: uuid::Uuid,
        last_page_number: i32,
    },

    #[error("Une réaction préalable est obligatoire pour révéler les métadonnées de cette page")]
    ReactionRequiredForReveal,

    #[error("Cette impression a déjà reçu une réaction")]
    AlreadyReacted,

    #[error("Continuation invalide : {0}")]
    InvalidContinuation(String),

    #[error("Code d'invitation invalide ou introuvable : {0}")]
    InvalidInviteCode(String),

    #[error("Code d'invitation épuisé ou expiré : {0}")]
    InviteCodeExpired(String),

    #[error("Erreur de base de données : {0}")]
    DatabaseError(String),
}
