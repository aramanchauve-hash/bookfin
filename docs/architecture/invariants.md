# Invariants Architecturaux & Matrice des Signaux Invisibles de Bookfin

Ce document formalise les règles invariantes non négociables et la matrice de confiance des signaux de lecture de l'infrastructure Bookfin.

---

## 1. Les 10 Invariants Architecturaux Absolus

### Invariant 1 : Priorité absolue à la lecture
Bookfin est avant tout une application de lecture littéraire. Sa vocation première n'est ni la recommandation algorithmique, ni l'optimisation de l'engagement compulsif, ni la gamification. L'expérience fondamentale est immédiate : ouvrir l'application, lire une page de littérature choisie sans friction, puis décider en toute liberté.

### Invariant 2 : Indépendance stricte de la Page vis-à-vis du terminal
Une `Page` métier est une unité textuelle stable et calibrée en tokens/mots littéraires indépendamment de tout écran ou périphérique. Le client mobile est le seul responsable de la mise en page typographique (taille de police, interlignage, marges, scrolling). Une page n'est jamais redimensionnée ou découpée en fonction de la résolution ou du viewport d'un client.

### Invariant 3 : Découverte aléatoire uniforme (`RANDOM_PAGE`)
Le feed principal sert une page aléatoire uniforme parmi l'ensemble des pages actives et éligibles de la langue du lecteur. Les pages déjà servies à l'utilisateur sont strictement exclues via un anti-join indexé `NOT EXISTS (SELECT 1 FROM page_impressions WHERE user_id = $1 AND page_id = p.id)`. Le système n'applique aucun biais de popularité ni filtre à bulles.

### Invariant 4 : Curation au niveau des Œuvres, jamais des Pages
La curation éditoriale s'effectue exclusivement à l'échelle des œuvres (`works`) et des éditions (`editions`). Aucune sélection artificielle de "meilleures pages" ou de "citations accrocheuses" n'est effectuée sur les pages individuelles. Chaque page d'une œuvre retenue est éligible au feed dans son intégrité textuelle.

### Invariant 5 : Anonymat pré-réaction strict (Blind Reading)
Toute page servie dans le feed anonyme masque rigoureusement le titre de l'œuvre, l'auteur, l'éditeur, l'année, ainsi que les identifiants internes d'œuvre et d'édition. Le contrat JSON d'API (`FeedPageDto`) ne transmet aucun de ces champs avant que le lecteur n'ait soumis une réaction explicite. La révélation (`/api/v1/pages/{id}/reveal`) est strictement interdite (HTTP 403) sans réaction préalable enregistrée.

### Invariant 6 : Découplage orthogonal entre Préférence et Navigation
L'axe de Préférence (`LIKE` / `DISLIKE`) et l'axe de Navigation (`CONTINUE_BOOK` / `RANDOM_PAGE`) sont totalement orthogonaux et indépendants :
- Un lecteur peut aimer une page tout en voulant découvrir un autre livre (`LIKE` + `RANDOM_PAGE`).
- Un lecteur peut être intrigué par un récit sombre ou déroutant et vouloir continuer la lecture malgré un sentiment mitigé (`DISLIKE` + `CONTINUE_BOOK`).
Le système n'infère jamais la navigation à partir du like, ni le like à partir de la continuation.

### Invariant 7 : Immutabilité absolue des Pages servies
Dès lors qu'une page a fait l'objet d'une impression historique ou d'une réaction dans la base de données, son contenu textuel (`content`), son hash d'intégrité SHA-256 (`content_hash`), son numéro de page (`page_number`) et son édition (`edition_id`) sont strictement non mutables. Un déclencheur PostgreSQL au niveau base (`trg_prevent_page_content_mutation`) lève une exception bloquante (`ImmutablePageViolation`) en cas de tentative d'UPDATE. Toute correction textuelle requiert une nouvelle version d'édition (`version = version + 1`).

### Invariant 8 : Idempotence réseau et unicité de réaction par impression
Chaque consultation génère une impression traçable (`page_impressions`). La réaction d'un lecteur requiert obligatoirement une impression valide préalable. Une impression ne peut être réagie qu'une seule fois (contrainte d'unicité partielle PostgreSQL `uq_reactions_impression_id`). Les réémissions réseau (doubles frappes, reconnexions) transmettent le même `event_id` ou `client_request_id` et sont résolues de manière strictement idempotente sans duplication ni erreur 500.

### Invariant 9 : Continuité séquentielle causale et détection de fin d'édition
L'action de navigation `CONTINUE_BOOK` lie de manière causale et vérifiable l'impression suivante à son impression parente (`parent_impression_id`). La chaîne causale ($P_{142} \to P_{143} \to P_{144}$) est reconstructible de manière déterministe via CTE récursive. Lorsque le lecteur atteint la dernière page d'une édition, le domaine rejette la continuation avec une erreur typée explicite `DomainError::EndOfEdition { edition_id, last_page_number }` (HTTP 404), invitant le lecteur à explorer une nouvelle page aléatoire.

### Invariant 10 : Confidentialité absolue et isolation des lecteurs
Les signaux de lecture et réactions sont strictement confinés à l'espace privé de chaque lecteur. Aucune API publique n'expose les likes, dislikes, temps de lecture ou historiques d'autres utilisateurs. Les calculs d'affinité mutuelle n'interviennent qu'en aval sous forme de faits agrégés sans jamais révéler les profils de lecture individuels.

---

## 2. Matrice des Signaux Invisibles de Lecture

Le tableau ci-dessous formalise l'origine, la précision, les bornes de sécurité, le niveau de confiance et l'usage de chaque métrique de lecture collectée.

| Signal | Origine | Unité / Type | Bornes & Plafonds | Risque de falsification client | Confiance serveur | Rôle dans la validation |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **Temps d'affichage (`dwell_time_ms`)** | Client (Chrono front-end) | Millisecondes (`i32`) | $\ge 0$, assaini à $\le 1\,800\,000$ ms (30 min) | **Élevé** (Horloge modifiable, timer manipulable par script) | **Faible** (Indicatif, non suffisant seul) | Validation croisée : rejet si $< \text{seuil minimal}$ ou si vitesse de lecture WPM irréaliste ($> 1\,200$ WPM). Assaini en cas d'oubli de l'application en veille. |
| **Progression de défilement (`scroll_depth_percent`)** | Client (DOM / Scroll listener) | Pourcentage (`f32`) | Clampé strictement entre $0.0\%$ et $100.0\%$ | **Moyen** (Simulation d'événements de scroll) | **Moyenne** (Vérifiable si plusieurs paliers atteints) | Condition nécessaire pour les pages excédant le viewport : seuil typique $\ge 70\%$. Pour les pages tenant intégralement sur l'écran, le scroll n'est pas pénalisé. |
| **Vitesse de lecture (`reading_speed_wpm`)** | Serveur / Dérivé | Mots par minute (`i32`) | $0 \le \text{WPM} \le 1\,200$ | **Moyen** (Dérivé de `dwell_time` client) | **Moyenne** (Calculé par le serveur à partir du `token_count` certifié) | Filtre anti-fraude : un lecteur humain moyen lit entre 150 et 350 WPM. Tout temps impliquant $> 1\,200$ WPM invalide la lecture qualifiée. |
| **Horodatage de distribution (`served_at`)** | Serveur (PostgreSQL `NOW()`) | Timestamp UTC (`timestamptz`) | Monotone serveur | **Nul** (Généré exclusivement par l'infrastructure backend) | **Absolue (100%)** | **Fondation de la validation temporelle** : le serveur mesure l'écart réel $\Delta t = T_{\text{réaction}} - T_{\text{distribution}}$. Toute réaction soumise avec $\Delta t < \text{seuil minimal}$ est rejetée, indépendamment du chronomètre client. |
| **Horodatage d'arrivée (`created_at`)** | Serveur (PostgreSQL `NOW()`) | Timestamp UTC (`timestamptz`) | Monotone serveur | **Nul** (Généré par le serveur à l'écriture) | **Absolue (100%)** | Permet d'horodater de manière inaltérable l'événement de réaction pour les calculs d'affinité temporelle. |
| **Identifiant d'impression (`impression_id`)** | Serveur (Généré au feed) | UUIDv4 | Clé étrangère vers `page_impressions` | **Faible** (UUID cryptographiquement imprévisible) | **Absolue (100%)** | Garantit qu'une réaction découle directement d'un extrait effectivement servi à ce lecteur précis. Clé d'idempotence stricte. |
| **Identifiant d'événement (`event_id`)** | Client (Idempotency Key) | UUIDv4 | Doit être un UUID valide | **Faible** (Généré par le client pour dédupliquer les requêtes réseau) | **Élevée** (Utilisé pour ignorer les réémissions sans dupliquer) | Clé de déduplication réseau : deux soumissions successives avec le même `event_id` renvoient le même état de succès sans créer d'enregistrement supplémentaire. |
| **Chaîne causale (`parent_impression_id`)** | Client (passé en requête de continuation) | UUIDv4 | Clé étrangère vers `page_impressions` | **Faible** (Vérifié côté serveur : doit correspondre au même `user_id` et à la même page) | **Absolue (100%)** | Permet la reconstruction déterministe des sessions de lecture continue ($P_1 \to P_2 \to \dots \to P_n$) et l'analyse de rétention sans cookies intrusifs. |

