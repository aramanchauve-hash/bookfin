# ADR 0003 : Pipeline de Lecture, Deux Axes Indépendants, Continuation et Signaux Invisibles

## Statut
Validé / Accepté

## Contexte
Le geste de lecture ne se réduit pas à une note binaire « j'aime / je n'aime pas ». Un lecteur peut apprécier une page tout en souhaitant continuer à explorer d'autres horizons au hasard. Inversement, une page rude ou déconcertante peut susciter la curiosité de découvrir la suite de l'œuvre. Confondre l'évaluation stylistique et l'intention de navigation introduisait un biais majeur dans les modèles d'engagement et d'affinité.

De plus, contraindre le lecteur à un défilement obligatoire de 75% même lorsque le texte tient intégralement dans son écran créait une friction indésirable et des rejets infondés.

## Décision

### 1. Deux Axes Strictement Indépendants
La réaction du lecteur à une page est désormais modélisée sur deux dimensions orthogonales :
1. **Axe Préférence (`ReactionType`)** :
   - `Like` / `Dislike` (ainsi que `Skip` ou `Save`).
   - Cet axe capture le ressenti esthétique et littéraire face au texte.
   - Les métadonnées de l'œuvre (auteur, titre, traducteur, date) restent invisibles avant l'expression de ce choix.
2. **Axe Navigation (`NavigationAction`)** :
   - `ContinueBook` : L'utilisateur désire lire la page immédiatement suivante de cette même édition (`page_sequence_number + 1`).
   - `RandomPage` : L'utilisateur souhaite retourner au hasard du corpus et recevoir une nouvelle page aléatoire.
   - Cet axe est indépendant de la préférence : tous les couples sont valides (`(Like, ContinueBook)`, `(Like, RandomPage)`, `(Dislike, ContinueBook)`, `(Dislike, RandomPage)`).

### 2. Mesure de la Continuation et Traçabilité
Pour capturer l'immersion dans une œuvre :
- **Profondeur de continuation (`continuation_depth`)** : Entier initialisé à 0 lors d'un tirage aléatoire, puis incrémenté de 1 à chaque page consécutivement enchaînée ($0 \rightarrow 1 \rightarrow 2 \dots$).
- **Généalogie de lecture** : Chaque impression consécutive référence son `parent_impression_id` ainsi que son `root_page_id` (la page d'entrée initiale).
- **Reprise de lecture (`book_resumed`)** : Booléen signalant si le lecteur reprend une œuvre déjà entamée lors d'une session antérieure.

### 3. Exploitation des Faits de Continuation pour les Affinités Émergentes
Les faits de continuation (nombre de pages consécutives lues, reprise ultérieure d'un livre) constituent des signaux d'engagement qualitatifs très puissants.
- **Principe d'architecture** : Ces faits sont rigoureusement collectés et stockés dans `page_impressions` et `reactions`.
- **Neutralité de pondération** : Aucun poids numérique arbitraire n'est fixé de manière anticipée dans la formule de Jaccard/Wilson. Le modèle conserve ces données brutes pour un étalonnage futur fondé sur des observations réelles.

### 4. Collecte des Signaux Invisibles de Lecture
Pour qualifier la validité et la sincérité de la lecture sans perturber le lecteur, les métriques suivantes sont enregistrées de façon transparente :
- `active_reading_time_ms` : Temps effectif passé avec l'application active au premier plan.
- `scroll_depth` : Ratio maximal de défilement atteint ($0.0 \dots 1.0$).
- `bottom_reached` : Booléen attestant que le lecteur a atteint le bas du texte.
- `scroll_back` : Booléen signalant une relecture ou un retour en arrière dans la page.
- `navigation_action` : Choix explicite de navigation opéré.
- `is_bookmarked` : Marquage éventuel de la page.

### 5. Assouplissement du Garde-Fou de Défilement (Flexible Scroll)
La condition de rejet pour défilement insuffisant a été rendue contextuelle :
- **Si le contenu ne déborde pas de l'écran (`content_overflows = false`)** : Le texte est entièrement lisible d'emblée. Le seuil de défilement minimal (75%) ne s'applique pas ; seul le temps de lecture minimal (4 000 ms) est exigé.
- **Si le contenu déborde (`content_overflows = true`)** : La lecture est validée si le lecteur a atteint le bas (`bottom_reached = true`) OU si son ratio de défilement atteint au moins 75% (`scroll_depth >= 0.75`).
- Les lectures instantanées bot/swipe rapide (< 4 000 ms) restent rejetées dans tous les cas.

