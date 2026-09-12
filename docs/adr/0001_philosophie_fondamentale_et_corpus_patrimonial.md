# ADR 0001 : Philosophie Fondamentale de Bookfin et Corpus Patrimonial

## Statut
Validé / Accepté

## Contexte
Bookfin est avant tout une **application de lecture littéraire immédiate**.
Sa raison d'être première n'est ni la recommandation algorithmique agressive, ni la création prématurée d'un réseau social, ni l'optimisation artificielle de métriques d'engagement (dopamine loops, notifications invasives).

Le cas d'usage canonique est sobre et épuré :
> Un lecteur est dans les transports (métro, train) ou en pause. Il saisit son téléphone, ouvre Bookfin, et une page de véritable littérature apparaît instantanément. Il la lit.

Si la page l'interpelle, il peut choisir de poursuivre la lecture de l'œuvre ou de découvrir une nouvelle page issue du hasard du corpus. L'émergence d'affinités entre lecteurs et la découverte sociale ne constituent qu'un **sous-produit naturel, différé et optionnel** d'une pratique de lecture accumulée dans le temps.

## Décision

### 1. Primauté de la Lecture Immédiate
L'expérience utilisateur priorise l'immersion textuelle :
- Aucun écran intermédiaire, aucun questionnaire de profilage obligatoire au premier lancement.
- Présentation anonymisée du texte au premier regard : le texte vaut par lui-même (style, voix, rythme) avant d'être jugé sur la notoriété de son auteur ou de son titre.
- Révélation des métadonnées (titre, auteur, traducteur, année) uniquement après que le lecteur a interagi avec la page.

### 2. Principe : « Curation des Œuvres + Aléatoire des Pages »
Le corpus littéraire de Bookfin repose sur une dialectique rigoureuse :
- **En amont (Curation éditoriale des Œuvres)** : Sélection exigeante d'éditions patrimoniales libres de droits ou sous licence ouverte, dotées d'une haute tenue littéraire, de traductions de référence et d'un appareil éditorial soigné.
- **En aval (Hasard des Pages)** : Tirage aléatoire uniforme parmi les pages éligibles du corpus, sans pondération biaisée par la popularité ou la brièveté de l'œuvre.

### 3. Rectification Formelle : Égalité de chance entre les pages éligibles
La formulation précédente mentionnant une *« équidistribution absolue des chances pour chaque œuvre »* a été formellement révoquée.
- **Règle retenue** : **Égalité de chance entre toutes les pages éligibles**.
- **Justification** : Si chaque œuvre avait une probabilité égale d'apparaître, une page d'un recueil de 30 pages aurait 30 fois plus de chances d'apparaître qu'une page de *Guerre et Paix* (1 200 pages). L'expérience de « feuilleter une gigantesque bibliothèque mondiale » exige que chaque page ait sa chance propre d'être ouverte par le lecteur.

## Conséquences
- L'infrastructure de sélection (`PageRepository`, `random_key`) attribue une distribution pseudo-aléatoire uniforme sur l'entité `Page`.
- Les filtres de sélection respectent les préférences linguistiques ordonnées de l'utilisateur (`user_language_preferences`) et excluent les pages déjà consultées.
- Le dimensionnement initial se focalise sur la fluidité d'affichage du texte et la fiabilité des signaux de lecture.

