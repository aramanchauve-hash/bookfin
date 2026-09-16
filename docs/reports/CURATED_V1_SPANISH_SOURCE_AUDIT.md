# Curated V1 — audit de qualité des sources espagnoles

Date : 2026-09-14  
Périmètre : les 21 œuvres espagnoles `ACCEPTED` de Curated V1, en lecture seule.

## Décision

| Recommandation | Œuvres |
| --- | ---: |
| KEEP SOURCE | 11 |
| KEEP + LIGHT NORMALIZATION | 10 |
| REPLACE SOURCE | 0 |

Les 21 statuts éditoriaux restent **ACCEPTED**. Cet audit ne modifie ni les
sources, ni les JSON/pages/hashes/verrou, ni Railway, EAS, le renderer ou la
pagination.

`KEEP + LIGHT NORMALIZATION` ne signifie pas de réécrire la littérature : les
corrections proposées ne visent que des nœuds HTML structurels explicitement
identifiés (`class="toc"`, `class="pagenum"`, en-têtes Gutenberg), avant la
création des blocs. Le paginator ne doit pas être modifié.

## Méthode et provenance

Chaque œuvre a été comparée à trois couches : le HTML stocké dans
`sources/es`, le document `normalized/es`, puis les pages `pages/es`, avec un
échantillon de page de début, milieu et fin. Le script
`scripts/scan_spanish_corpus_anomalies.py` produit les mesures et extraits de
façon déterministe. `scripts/test_spanish_source_audit.py` verrouille les
signatures connues et la couverture des 21 œuvres.

La colonne « source actuelle » décrit **les octets réellement archivés** : elle
est lue dans `curated_v1/integrity_audit_report.json`, plutôt que dans le
fichier de présélection, dont certaines URL Wikisource ont été remplacées par
un master Gutenberg lors de la construction V1.

Abréviations : `S` = HTML source, `N` = JSON normalisé, `P` = pages. Les
comptages `S/N/P` de marqueurs correspondent à `[Pg N]`, `[Page N]` ou `p. N`.

## Classement œuvre par œuvre

| Work ID | Titre | Source actuelle | Format | Problèmes observés (S → N → P) | Propreté source | Risque normalizer | Recommandation | Raison / remplacement candidat | Statut |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `es-cervantes-quijote` | Don Quijote de la Mancha | Gutenberg #2000 | HTML Gutenberg | Table des matières en `<div/ol class="toc">`; 137 jonctions minuscules/majuscules dans N et P (`…ManchaTasa…`, `letrasDonde`). | 3/5 | Faible | **KEEP + LIGHT NORMALIZATION** | Ignorer le conteneur CSS `toc` avant lecture ; le corps reste intact. Aucun remplacement requis. | ACCEPTED, inchangé |
| `es-cervantes-rinconete-y-cortadillo` | Rinconete y Cortadillo | Gutenberg #61202, anthologie | HTML Gutenberg | S 388 / N 23 / P 23 marqueurs `p. N`; les pages cibles restent correctement bornées. | 3/5 | Faible | **KEEP + LIGHT NORMALIZATION** | Retirer les spans `pagenum`; conserver le découpage existant de l’anthologie. | ACCEPTED, inchangé |
| `es-cervantes-la-gitanilla` | La gitanilla | Gutenberg #61202, anthologie | HTML Gutenberg | S 388 / N 13 / P 13 marqueurs `p. N`; aucune jonction suspecte. | 3/5 | Faible | **KEEP + LIGHT NORMALIZATION** | Même règle `pagenum`, sans toucher au texte. | ACCEPTED, inchangé |
| `es-cervantes-el-licenciado-vidriera` | El licenciado Vidriera | Gutenberg #61202, anthologie | HTML Gutenberg | S 388 / N 4 / P 4 marqueurs `p. N`; aucune jonction suspecte. | 3/5 | Faible | **KEEP + LIGHT NORMALIZATION** | Même règle `pagenum`, sans toucher au texte. | ACCEPTED, inchangé |
| `es-cervantes-el-celoso-extremeno` | El celoso extremeño | Gutenberg #61202, anthologie | HTML Gutenberg | S 388 / N 6 / P 6 marqueurs `p. N`; aucune jonction suspecte. | 3/5 | Faible | **KEEP + LIGHT NORMALIZATION** | Même règle `pagenum`, sans toucher au texte. | ACCEPTED, inchangé |
| `es-cervantes-el-coloquio-de-los-perros` | El coloquio de los perros | Gutenberg #61202, anthologie | HTML Gutenberg | S 388 / N 18 / P 18 marqueurs `p. N`; aucune jonction suspecte. | 3/5 | Faible | **KEEP + LIGHT NORMALIZATION** | Même règle `pagenum`, sans toucher au texte. | ACCEPTED, inchangé |
| `es-quevedo-los-suenos` | Los sueños | Gutenberg #65999 | HTML Gutenberg | S 322 / N 322 / P 322 marqueurs `[Pg N]`; l’index est concaténé (`ÍNDICE PÁG.IntroducciónVII…`). | 3/5 | Faible | **KEEP + LIGHT NORMALIZATION** | Exclure `pagenum` et le conteneur de table des matières ; aucune substitution de source nécessaire. | ACCEPTED, inchangé |
| `es-quevedo-el-buscon` | Historia de la vida del Buscón | Gutenberg #32315 | HTML Gutenberg | Aucun marqueur ou collage structurel détecté dans N/P. | 5/5 | Aucun | **KEEP SOURCE** | Source et chaîne de blocs propres dans les trois échantillons. | ACCEPTED, inchangé |
| `es-larra-el-doncel` | El doncel de don Enrique el Doliente | Gutenberg, 4 tomes concaténés (53587–53590) | HTML Gutenberg | S 627 / N 627 / P 627 marqueurs; 4 en-têtes « The Project Gutenberg eBook… » subsistent dans N/P, un par tome. | 2/5 | Moyen | **KEEP + LIGHT NORMALIZATION** | Le matériau de chaque tome est bon, mais l’acquisition concaténée exige l’exclusion de chaque header/footer Gutenberg et des `pagenum`. Ne pas remplacer sans besoin. | ACCEPTED, inchangé |
| `es-pardo-bazan-los-pazos-de-ulloa` | Los pazos de Ulloa | Gutenberg #18005 | HTML Gutenberg | Aucun marqueur ou collage structurel détecté dans N/P. | 5/5 | Aucun | **KEEP SOURCE** | Échantillons début/milieu/fin propres. | ACCEPTED, inchangé |
| `es-galdos-misericordia` | Misericordia | Gutenberg #21831 | HTML Gutenberg | Aucun marqueur ou collage structurel détecté dans N/P. | 5/5 | Aucun | **KEEP SOURCE** | Échantillons début/milieu/fin propres. | ACCEPTED, inchangé |
| `es-galdos-dona-perfecta` | Doña Perfecta | Gutenberg #15725 | HTML Gutenberg | Aucun marqueur ou collage structurel détecté dans N/P. | 5/5 | Aucun | **KEEP SOURCE** | Échantillons début/milieu/fin propres. | ACCEPTED, inchangé |
| `es-unamuno-niebla` | Niebla | Gutenberg #49836 | HTML Gutenberg | S 296 / N 297 / P 297 marqueurs `[Pg N]`, dont `[Pg 13]`; la source est `<span class="pagenum" id="Page_13">[Pg 13]</span>`. | 4/5 | Faible | **KEEP + LIGHT NORMALIZATION** | Exclure exactement les spans `pagenum`. N est la première couche fautive ; P le recopie. | ACCEPTED, inchangé |
| `es-unamuno-san-manuel-bueno-martir` | San Manuel Bueno, mártir | Wikisource | HTML Wikisource | Aucun marqueur ou collage structurel détecté dans N/P. | 5/5 | Aucun | **KEEP SOURCE** | Source et chaîne de blocs propres. | ACCEPTED, inchangé |
| `es-valle-inclan-sonata-otono` | Sonata de otoño | Gutenberg #37537 | HTML Gutenberg | Aucun marqueur ou collage structurel détecté dans N/P. | 5/5 | Aucun | **KEEP SOURCE** | Source et chaîne de blocs propres. | ACCEPTED, inchangé |
| `es-valle-inclan-sonata-primavera` | Sonata de primavera | Wikisource | HTML Wikisource | Un caractère éditorial source : `contemPlando`, présent identiquement dans N/P. Pas un défaut de frontière HTML. | 4/5 | Aucun | **KEEP SOURCE** | Conserver le source ; traiter seulement dans une future politique d’errata éditoriale, pas par le normalizer. | ACCEPTED, inchangé |
| `es-valle-inclan-luces-de-bohemia` | Luces de bohemia | Gutenberg #68745 | HTML Gutenberg | S 285 / N 216 / P 216 marqueurs; `<p class="rol"><span class="pagenum">p. 14</span>MAX</p>` devient `p. 14MAX`. | 4/5 | Faible | **KEEP + LIGHT NORMALIZATION** | Écarter `pagenum` avant l’agrégation des spans ; `MAX` redevient alors propre. Les rôles ne doivent pas être masqués dans le renderer. | ACCEPTED, inchangé |
| `es-quiroga-a-la-deriva` | A la deriva | Wikisource | HTML Wikisource | Aucun marqueur ou collage structurel détecté dans N/P. | 5/5 | Aucun | **KEEP SOURCE** | Source et chaîne de blocs propres. | ACCEPTED, inchangé |
| `es-quiroga-el-almohadon-de-plumas` | El almohadón de plumas | Wikisource | HTML Wikisource | Aucun marqueur ou collage structurel détecté dans N/P. | 5/5 | Aucun | **KEEP SOURCE** | Source et chaîne de blocs propres. | ACCEPTED, inchangé |
| `es-quiroga-la-insolacion` | La insolación | Wikisource | HTML Wikisource | Aucun marqueur ou collage structurel détecté dans N/P. | 5/5 | Aucun | **KEEP SOURCE** | Source et chaîne de blocs propres. | ACCEPTED, inchangé |
| `es-clarin-adios-cordera` | ¡Adiós, Cordera! y otros cuentos | Wikisource | HTML Wikisource | Aucun marqueur ou collage structurel détecté dans N/P. | 5/5 | Aucun | **KEEP SOURCE** | Source et chaîne de blocs propres. | ACCEPTED, inchangé |

## Chaîne causale des cas prioritaires

| Priorité | Œuvre | Source | JSON normalisé | Pages | Couche responsable |
| --- | --- | --- | --- | --- | --- |
| 1 | Larra — *El doncel* | Quatre documents Gutenberg complets concaténés, avec 627 marques de pagination. | Les en-têtes Gutenberg de chacun des tomes et les marques survivent. | Réplique exactement ces blocs. | Acquisition/normalizer, jamais paginator. |
| 2 | Cervantes — *Quijote* | `<div class="toc">` et `<ol class="toc">` au début. | Le parser ne traite pas `toc` comme classe exclue : toute la table devient un paragraphe, avec 137 collages. | Réplique ce premier grand bloc. | Normalizer. |
| 3 | Quevedo — *Los sueños* | Pagination imprimée sous `<span class="pagenum">`; index imprimé. | Les 322 marques et l’index survivent. | Réplique exactement ces éléments. | Normalizer. |
| 4 | Unamuno — *Niebla* | Exemple direct : `<span class="pagenum" id="Page_13">[Pg 13]</span>`. | `[Pg 13]` est conservé. | `[Pg 13]` est conservé. | Normalizer. |
| 5 | Valle-Inclán — *Luces* | Exemple direct : `<span class="pagenum" id="Page_14">p. 14</span>MAX`. | `p. 14MAX`. | `p. 14MAX`. | Normalizer. |

## Correction ultérieure, limitée et sûre

Ordre recommandé, lorsqu’une passe de reconstruction sera explicitement
autorisée :

1. Ajouter `toc` aux classes HTML structurelles exclues et ne supprimer que
   ces conteneurs (Quijote, index de *Los sueños*).
2. Ignorer le contenu des spans portant la classe `pagenum`, y compris quand
   ils sont imbriqués dans un `p` de rôle (Niebla, Luces, Cervantes, Quevedo,
   Larra).
3. Pour la concaténation Larra, redémarrer proprement l’exclusion des wrappers
   `pg-header`/`pg-footer` pour chaque document HTML concaténé.
4. Re-générer uniquement après validation éditoriale, puis re-auditer les
   hashes et les pages ; cette étape est volontairement hors de cette tâche.

Cette règle est générique, explicable et non littéraire. Elle ne doit pas
utiliser une expression régulière sur le texte courant ni modifier les mots
source. Le caractère `contemPlando` est volontairement exclu : c’est un errata
du matériau source, pas une frontière que le normalizer peut inférer sûrement.

## Commandes de vérification

```powershell
cd C:\Users\arama\Dev\Bookfin
python scripts\test_spanish_source_audit.py
python scripts\scan_spanish_corpus_anomalies.py
```

Résultat attendu : `Spanish Curated V1 source audit checks passed (21 works).`
