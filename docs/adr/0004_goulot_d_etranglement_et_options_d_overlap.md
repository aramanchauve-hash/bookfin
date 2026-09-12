# ADR 0004 : Problématique du Goulot d'Étranglement d'Overlap et Options Architecturales

## Statut
Option architecturale / Hypothèse documentée (Non gravée comme décision produit définitive)

## Contexte
Dans une bibliothèque de taille réelle comptant plus de 100 000 pages littéraires, si chaque utilisateur pioche de manière strictement aléatoire et uniforme parmi l'ensemble du corpus, la probabilité que deux lecteurs $A$ et $B$ tombent par hasard sur la même page (co-lecture) avec un historique modeste (ex: 50 à 100 pages chacun) est mathématiquement quasi nulle :
$$P(\text{intersection} \neq \emptyset) \approx 1 - \exp\left(-\frac{N_A \cdot N_B}{C}\right)$$
Pour $N_A = N_B = 100$ et $C = 100\,000$, la probabilité de croiser une seule page commune est inférieure à $9.5\%$, et la probabilité d'obtenir une intersection statistiquement significative ($\ge 5$ pages communes) tend vers 0.

L'hypothèse d'une « piscine tournante d'œuvres actives » (*Active Sliding Epoch Pool*) avait été esquissée pour concentrer artificiellement les tirages aléatoires sur une sous-fraction du corpus. 

## Décision

### 1. Statut de l'Active Sliding Epoch Pool
L'Active Sliding Epoch Pool n'est **pas** une décision produit définitive ni une règle rigide de Bookfin. Il s'agit d'une **option architecturale documentée**, destinée à être évaluée lorsque le corpus complet sera injecté et que le rythme d'acquisition d'utilisateurs sera connu.

### 2. Comparaison Structurée de 6 Modèles d'Overlap
Pour traiter cette problématique sans dénaturer la philosophie littéraire de l'application, 6 modèles sont documentés et comparés :

| Modèle | Principe de Fonctionnement | Avantages | Inconvénients / Risques | Complexité |
| :--- | :--- | :--- | :--- | :--- |
| **1. Pur Aléatoire Uniforme** | Tirage équiprobable sur l'intégralité des pages éligibles du corpus mondial. | Fidélité absolue à la philosophie de sérendipité littéraire ; aucune manipulation algorithmique. | Overlap quasi nul au démarrage avec peu d'utilisateurs ; report de la découverte sociale à un horizon lointain. | Très faible ($O(1)$) |
| **2. Active Sliding Epoch Pool** | Restreindre le tirage aléatoire à une fenêtre glissante de $K$ œuvres (ex: 50 œuvres par semaine). | Augmente massivement l'overlap à court terme ; crée des résonances communes entre lecteurs simultanés. | Risque d'uniformiser l'expérience hebdomadaire ; sentiment de bibliothèque restreinte. | Moyenne (rotation cron/epoch) |
| **3. Clusters Linguistiques & Époques** | Tirage aléatoire au sein de strates (ex: siècle, mouvement littéraire, langue source). | Respecte la cohérence d'intérêt des lecteurs ; favorise des affinités ciblées sans enfermer. | Risque de segmentation précoce en silos étanches si les catégories sont trop étroites. | Faible à Moyenne |
| **4. Œuvres Piliers / Anchors** | 10 à 20% des tirages sont réservés à un groupe de classiques fondateurs partagés par tous. | Garantit une base de co-lecture minimale pour chaque nouvel arrivant sans brider la diversité. | Peut lasser les lecteurs qui tombent fréquemment sur les mêmes chefs-d'œuvre très connus. | Faible |
| **5. Propagation par Continuation** | Lorsqu'un utilisateur active `ContinueBook`, les pages suivantes de l'œuvre deviennent prioritaires pour les autres. | Émergence naturelle des livres captivants ; découle d'un acte de lecture sincère sans algorithme artificiel. | Effet « winner-takes-all » si un livre très lu vampirise les tirages des autres. | Moyenne |
| **6. Réservoir Hybride Adaptatif** | L'amplitude du sous-ensemble aléatoire s'adapte dynamiquement au nombre d'utilisateurs actifs quotidiens. | Optimise mathématiquement l'overlap : corpus resserré quand $DAU$ est faible, déverrouillage total à grande échelle. | Algorithme d'échantillonnage adaptatif plus complexe à maintenir et à expliquer. | Élevée |

### 3. Statut du Seuil de Maturité de Profil
Le seuil de maturité de profil fixé à **200 interactions** (`profile_maturity_threshold = 200`) est une **valeur par défaut de développement**, hautement configurable dans `ReadingValidationConfig`.
- Il évite de tirer des conclusions précipitées sur les goûts d'un lecteur au bout de quelques pages lues distraitement.
- Il pourra être ajusté dynamiquement selon les conclusions empiriques tirées du comportement des utilisateurs réels.

