# BOOKFIN — AUDIT D'INTÉGRITÉ DOCUMENTAIRE FINAL (CORPUS CURATED V1)
**Date** : 13 septembre 2026  
**Périmètre** : 73 œuvres | 11 217 pages Bookfin V2  
**Environnement** : Local (zéro écriture Railway, base intacte)  
**Décision finale requise** : Validation humaine préalable à tout import  

---

## 1. Bilan Global des Statuts Techniques d'Intégrité

Sur les 73 œuvres constituant le corpus Curated V1 :

| Statut Technique | Nombre | Pourcentage | Description |
| :--- | :---: | :---: | :--- |
| **VERIFIED** | **50** | **68.5 %** | Texte intégral, authentique, langue conforme, début/fin vérifiés, pagination continue. |
| **REVIEW_REQUIRED** | **11** | **15.1 %** | Texte authentique mais reliquat mineur de métadonnées Wikisource/TOC finale à épurer. |
| **STRUCTURE_ISSUE** | **3** | **4.1 %** | Appareil critique géant, recueil entier au lieu d'une nouvelle, ou lexique scolaire inclus. |
| **TRUNCATED** | **1** | **1.4 %** | Œuvre amputée (Volume 1 seul sur 2 volumes). |
| **SOURCE_MISMATCH** | **8** | **11.0 %** | Mauvais livre téléchargé (id Gutenberg incorrect ou texte en mauvaise langue). |
| **DUPLICATED** | **0** | **0.0 %** | Aucune duplication globale ou chapitre injecté deux fois. |
| **TOTAL** | **73** | **100 %** | |

> **Conclusion immédiate** : **Le corpus NE DOIT PAS être importé en production dans cet état.**  
> 8 œuvres (11 %) ne contiennent absolument pas le texte annoncé, 1 roman majeur est tronqué de moitié, et 3 œuvres contiennent des sections massives hors-texte.

---

## 2. Explication Objective des Outliers Prioritaires

| Œuvre annoncée | Est. | Réel V2 | Écart | Diagnostic réel et cause objective |
| :--- | :---: | :---: | :---: | :--- |
| **A Sentimental Journey** (*L. Sterne*) | ~170 | 500 | **+194 %** | **SOURCE_MISMATCH** : Gutenberg #107 correspond en réalité au roman *Far from the Madding Crowd* de Thomas Hardy. Sterne n'a jamais été extrait. |
| **Table-Talk** (*W. Hazlitt*) | ~390 | 40 | **-89.7 %** | **SOURCE_MISMATCH** : Gutenberg #4335 est en réalité un court traité d'économie politique de T.R. Malthus (*Importation of Foreign Corn*). |
| **El doncel de don Enrique** (*M.J. de Larra*) | ~395 | 61 | **-84.6 %** | **SOURCE_MISMATCH** : Gutenberg #51184 est *Inside Earth* de Poul Anderson (nouvelle de SF en anglais issue du magazine Galaxy de 1951). |
| **Sonata de otoño** (*R. del Valle-Inclán*) | ~92 | 28 | **-69.7 %** | **SOURCE_MISMATCH** : Gutenberg #37537 est *San Francisco in Ruins* de J.D. Givens (album photo militaire en anglais sur le tremblement de terre de 1906). |
| **The Private Papers of Henry Ryecroft** (*G. Gissing*) | ~230 | 88 | **-61.7 %** | **SOURCE_MISMATCH** : Gutenberg #2304 est *Legends and Lyrics* d'Adelaide Anne Procter (poèmes victoriens). |
| **Le Diable boiteux** (*A.-R. Lesage*) | ~260 | 100 | **-61.5 %** | **SOURCE_MISMATCH** : Gutenberg #33434 est *The Squirrels and Other Animals* de George Waring (histoire naturelle pour enfants en anglais). |
| **The Portrait of a Lady** (*H. James*) | ~800 | 424 | **-47.0 %** | **TRUNCATED** : Gutenberg #2833 ne contient que le Volume 1 (chapitres I à XXVII). Le roman compte 55 chapitres ; le Volume 2 (Gutenberg #2834) n'a pas été rattaché. |
| **Le Journal d'une femme de chambre** (*O. Mirbeau*) | ~400 | 231 | **-42.2 %** | **SOURCE_MISMATCH** : Gutenberg #43170 est *Prentice Hugh* de Frances Mary Peard (roman historique en anglais sur la cathédrale d'Exeter). |
| **Essays of Elia** (*C. Lamb*) | ~345 | 728 | **+111 %** | **STRUCTURE_ISSUE** : Gutenberg #10343 contient l'intégralité des œuvres éditées par E.V. Lucas (notes critiques de 400 pages et index alphabétique exhaustif A-Z). |
| **Don Quijote de la Mancha** (*M. de Cervantes*) | ~1250 | 1365 | **+9.2 %** | **VERIFIED** : Don Quijote est complet (Partie 1 et Partie 2, 126 chapitres au total). L'estimation initiale à 890 pages ne considérait qu'une seule partie ou un découpage dense. |

---

## 3. Autres Anomalies Critiques Détectées

- **fr-merimee-carmen** : **SOURCE_MISMATCH** (Gutenberg #2465 est la traduction anglaise de Lady Mary Loyd). De plus, un bug de segmentation a généré une **page 42 totalement vide (0 bloc)**.
- **fr-maupassant-boule-de-suif** : **STRUCTURE_ISSUE** (Gutenberg #10746 contient l'intégralité du recueil de 18 nouvelles d'Ollendorff, soit 141 pages au lieu de la seule nouvelle éponyme ~50 p.).
- **es-galdos-dona-perfecta** : **STRUCTURE_ISSUE** (Gutenberg #15725 est une édition pour étudiants américains : préface, notes en anglais et 34 pages de dictionnaire de vocabulaire espagnol-anglais en fin de volume).

---

## 4. Audit des Formats Sources (EPUB vs HTML)

- **73 / 73 fichiers** dans `corpus/curated_v1/sources` sont des fichiers `.html`.
- **Raison technique** : Le script de téléchargement `build_curated_v1.py` ciblait explicitement le master XHTML Gutenberg (`pg{id}-images.html`) et la vue globale Wikisource (`Texte_entier`), afin d'alimenter directement `BookfinHtmlParser`.
- **Non-conformité documentaire** : Les fichiers `.epub` originaux n'ont pas été archivés côte-à-côte avec les fichiers HTML comme exigé par la règle de conservation.

---

## 5. Métriques des 11 217 Pages Bookfin V2

- **Nombre total de pages** : 11 217
- **Moyenne caractères / page** : 1 573.3 car.
- **Médiane caractères / page** : 1 604 car.
- **Percentile 5 (P5)** : 1 231 car.
- **Percentile 95 (P95)** : 1 775 car.
- **Minimum** : 0 car. (*page 42 de fr-merimee-carmen*, anomalie isolée)
- **Maximum** : 2 182 car. (*es-cervantes-quijote*, paragraphe long indivisible préservé sans troncature)
- **Doublons de hash au sein d'une même œuvre** : 0
- **Continuité des séquences** : 100 % (1 .. N sans saut)

---

## 6. Fixture Mobile Multilingue

La fixture mobile précédente (12 pages toutes issues de *Pride and Prejudice*) a été remplacée par une fixture équilibrée dans `corpus/curated_v1/mobile_preview_fixture.json` :
- **4 pages Anglais** : *Pride and Prejudice*, *Persuasion*, *The Picture of Dorian Gray*, *A Scandal in Bohemia* (prose, italique, heading, dialogue).
- **4 pages Français** : *La Chartreuse de Parme*, *Bel-Ami*, *À rebours*, *Histoires désobligeantes* (accents, dialogue avec guillemets et tirets, scene_break).
- **4 pages Espagnol** : *Don Quijote*, *Misericordia*, *Los pazos de Ulloa*, *El celoso extremeño* (ponctuation ¿ ¡, accents, ñ, dialogue).

---

## 7. Tests d'Intégrité Automatisés

Le script de tests unitaire `scripts/test_curated_v1_integrity.py` a été développé et validé (6 tests unitaires sans réseau, exécutés en 0.94 s) :
- Présence et validité des fichiers pour les 73 œuvres
- Intégrité stricte des hashes SHA-256
- Continuité des séquences de pagination
- Absence de page vide et de doublon sur les 50 œuvres VERIFIED
- Détection des anomalies connues (SOURCE_MISMATCH, volume tronqué, page vide)
- Équilibre de la fixture multilingue
