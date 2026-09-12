# ADR 0002 : Définition de la Page, Double Pagination et Idempotence d'Ingestion

## Statut
Validé / Accepté (avec section ouverte sur la pagination des sources continues)

## Contexte
Les formulations initiales de l'ingestion comportaient plusieurs limitations conceptuelles :
1. Une dépendance implicite envers les caractéristiques de l'écran mobile (estimations en nombre d'écrans de smartphone, fourchettes rigides de tokens/mots entre 180 et 450 tokens, calibrage selon la hauteur de viewport).
2. Une exigence artificielle imposant à chaque page de respecter les frontières phrastiques ou les unités narratives complètes.
3. Une confusion entre l'ordre de lecture dans Bookfin et la pagination d'origine de l'ouvrage physique.
4. Une ambiguïté sur le rôle du hash SHA-256 (`content_hash`), parfois interprété à tort comme un mécanisme de déduplication globale inter-éditions.

## Décision

### 1. Indépendance Totale envers le Terminal
Une `Page` dans Bookfin est une entité textuelle autonome, **strictement indépendante du terminal de restitution**.
- Aucune notion de dimension d'écran, de nombre d'écrans ou de hauteur de viewport n'est stockée ni calculée dans le domaine ou la base de données.
- Le client mobile (ou web) est l'unique responsable de la composition typographique, des sauts de ligne, du corps de police et du défilement éventuel.
- Selon la taille de l'écran, les réglages d'accessibilité de l'utilisateur ou la langue, une page peut s'afficher d'un seul bloc ou nécessiter un défilement vertical.

### 2. Suppression de la Contrainte des Frontières Phrastiques
Une page Bookfin réplique l'expérience authentique de **l'ouverture impromptue d'un livre physique à une page donnée**.
- La page n'a **pas** à commencer obligatoirement par le début d'une phrase, ni à se terminer à la fin d'un paragraphe ou d'un chapitre.
- Elle peut débuter au beau milieu d'une proposition ou s'achever sur une suspension textuelle, préservant la continuité brute du texte littéraire.

### 3. Double Système de Pagination
Chaque page enregistrée dans le système dispose de deux informations distinctes :
1. **`page_sequence_number` (int, non-null)** : Entier séquentiel strictement croissant (1, 2, 3...) définissant l'ordre de lecture rigoureux et continu au sein d'une édition spécifique. Il constitue la clé de la navigation séquentielle (« Continuer ce livre »).
2. **`source_page_number` (text, nullable)** : Chaîne textuelle représentant la pagination originale de la source de référence (ex: `"42"`, `"XII"`, `"IV-3"`). Cette valeur est informative et peut être nulle pour des sources nativement non paginées.

### 4. Sources Continues Non Paginées (EPUB, TXT) — Décision Ouverte
Pour les sources sans pagination imprimée préexistante :
- **Hypothèse de référence retenue** : Découpage déterministe typographique fondé sur un nombre cible de caractères ou de glyphes normalisés (ex: 1 500 à 2 000 caractères avec espace), garantissant la reproductibilité stricte de la séquence sans heuristiques sémantiques instables.
- **Alternative envisageable** : Découpage au niveau des balises structurelles de l'EPUB (`<p>`, `<div>`, `<section>`) avec coalescence des petits paragraphes.
- Cette décision reste ouverte et fera l'objet d'un sous-protocole d'ingestion avant le traitement des archives complètes.

### 5. Identité de la Page et Rôle de SHA-256
- **Identité métier** : L'identité d'une page est strictement circonscrite au triplet :
  $$\text{PageIdentity} = (\text{edition\_id}, \text{page\_sequence\_number}, \text{version})$$
- **Rôle de `content_hash`** : Le SHA-256 du texte normalisé (espaces fusionnés, encodage UTF-8 canonique) sert exclusivement :
  1. À la vérification d'intégrité des données à l'import.
  2. À l'idempotence de l'ingestion au niveau de la source (éviter d'insérer deux fois la même page lors d'une relance du job d'ingestion).
- **Non-déduplication inter-éditions** : Deux traductions différentes ou deux éditions distinctes d'un même texte partagent le même `work_id` mais possèdent des `edition_id` distincts et des pages distinctes, même si certains passages coïncident textuellement.

