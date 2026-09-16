# AUDIT QUALITATIF DU CORPUS BOOKFIN CURATED V1
**Rapport déterministe généré mécaniquement depuis les artefacts JSON réels**

- **Date de génération :** 2026-09-13 16:33:22 UTC
- **Nombre d'œuvres attendu (Lock) :** `73`
- **Nombre d'œuvres effectivement présent :** `73`
- **Concordance absolue des œuvres :** `OUI (100%)`

### Empreintes cryptographiques (SHA-256) des artefacts sources :
- `library_v1_candidates.json` : `4e8cb45e6a84d3927bbeb3616887e9d03c3c7e2de7a26e100be28dc675a157f6`
- `curated_v1.lock.json` : `f4994578ef217f2dbf1d3017207cea70ad8c2dc0288b3f065630e9ee330066c9`
- `corpus/curated_v1/manifest.json` : `9ed038bd75fa33fbd2b94ba2cd8ac2246bb3d70af58afcf9079f9f00fae58ee6`
- `integrity_audit_report.json` : `ab1a5c3211dfea88b9e692a355dcdfed62c471904c23a56257cfad67ea8e203e`

---

## 1 — SYNTHÈSE QUALITATIVE GLOBALE

| Statut Qualité | Nombre d'œuvres | Ratio | Signification opérationnelle |
|---|---|---|---|
| **`PASS`** | **70** | 95.9 % | Œuvre intègre, complète, authentique, pagination propre : prête pour production |
| **`REVIEW`** | **3** | 4.1 % | Œuvre authentique et complète, mais structure stylistique/théâtrale nécessitant validation visuelle |
| **`BLOCK`** | **0** | 0.0 % | Œuvre non-conforme (discordance de source, troncature de tome ou recueil parasite) : import interdit |
| **TOTAL** | **73** | **100.0 %** | Exactement les 73 œuvres de la décision éditoriale humaine |

---

## 2 — TABLEAU MÉCANIQUE DES 73 ŒUVRES DU CORPUS CURATED V1

*Ce tableau est produit par sérialisation directe des objets JSON du manifeste local sans intervention manuelle.*

| # | Work ID | Auteur & Titre | Langue | Type | Pages V2 | Statut | Diagnostic / Motif |
|---|---|---|---|---|---|---|---|
| 01 | `en-austen-pride-prejudice` | Jane Austen — *Pride and Prejudice* | EN | LONG_FORM | 443 | **`PASS`** | Texte intégral, authentique, pagination stable, absence de boilerplate. |
| 02 | `en-austen-persuasion` | Jane Austen — *Persuasion* | EN | LONG_FORM | 305 | **`PASS`** | Texte intégral, authentique, pagination stable, absence de boilerplate. |
| 03 | `en-johnson-rasselas` | Samuel Johnson — *The History of Rasselas, Prince of Abissinia* | EN | SHORT_WORK | 148 | **`PASS`** | Texte intégral, authentique, pagination stable, absence de boilerplate. |
| 04 | `en-sterne-sentimental-journey` | Laurence Sterne — *A Sentimental Journey Through France and Italy* | EN | LONG_FORM | 141 | **`PASS`** | Texte intégral, authentique, pagination stable, absence de boilerplate. |
| 05 | `en-browne-religio-medici` | Sir Thomas Browne — *Religio Medici* | EN | SHORT_WORK | 116 | **`PASS`** | Texte intégral, authentique, pagination stable, absence de boilerplate. |
| 06 | `en-browne-hydriotaphia` | Sir Thomas Browne — *Hydriotaphia, Urn Burial* | EN | SHORT_WORK | 58 | **`PASS`** | Texte intégral, authentique, pagination stable, absence de boilerplate. |
| 07 | `en-hazlitt-table-talk` | William Hazlitt — *Table-Talk* | EN | LONG_FORM | 591 | **`PASS`** | Texte intégral, authentique, pagination stable, absence de boilerplate. |
| 08 | `en-gissing-henry-ryecroft` | George Gissing — *The Private Papers of Henry Ryecroft* | EN | LONG_FORM | 230 | **`PASS`** | Texte intégral, authentique, pagination stable, absence de boilerplate. |
| 09 | `en-james-portrait-of-a-lady` | Henry James — *The Portrait of a Lady* | EN | LONG_FORM | 833 | **`PASS`** | Texte intégral, authentique, pagination stable, absence de boilerplate. |
| 10 | `en-james-turn-of-the-screw` | Henry James — *The Turn of the Screw* | EN | SHORT_WORK | 147 | **`PASS`** | Texte intégral, authentique, pagination stable, absence de boilerplate. |
| 11 | `en-james-daisy-miller` | Henry James — *Daisy Miller* | EN | SHORT_WORK | 79 | **`PASS`** | Texte intégral, authentique, pagination stable, absence de boilerplate. |
| 12 | `en-james-beast-in-the-jungle` | Henry James — *The Beast in the Jungle* | EN | SHORT_WORK | 65 | **`PASS`** | Texte intégral, authentique, pagination stable, absence de boilerplate. |
| 13 | `en-james-aspern-papers` | Henry James — *The Aspern Papers* | EN | SHORT_WORK | 136 | **`PASS`** | Texte intégral, authentique, pagination stable, absence de boilerplate. |
| 14 | `fr-mirbeau-journal-femme-chambre` | Octave Mirbeau — *Le Journal d'une femme de chambre* | FR | LONG_FORM | 420 | **`PASS`** | Texte intégral, authentique, pagination stable, absence de boilerplate. |
| 15 | `fr-maupassant-bel-ami` | Guy de Maupassant — *Bel-Ami* | FR | LONG_FORM | 381 | **`PASS`** | Texte intégral, authentique, pagination stable, absence de boilerplate. |
| 16 | `fr-maupassant-pierre-et-jean` | Guy de Maupassant — *Pierre et Jean* | FR | LONG_FORM | 174 | **`PASS`** | Texte intégral, authentique, pagination stable, absence de boilerplate. |
| 17 | `fr-maupassant-la-parure` | Guy de Maupassant — *La Parure* | FR | SHORT_WORK | 10 | **`PASS`** | Texte intégral, authentique, pagination stable, absence de boilerplate. |
| 18 | `fr-maupassant-la-ficelle` | Guy de Maupassant — *La Ficelle* | FR | SHORT_WORK | 8 | **`PASS`** | Texte intégral, authentique, pagination stable, absence de boilerplate. |
| 19 | `fr-maupassant-deux-amis` | Guy de Maupassant — *Deux amis* | FR | SHORT_WORK | 8 | **`PASS`** | Texte intégral, authentique, pagination stable, absence de boilerplate. |
| 20 | `fr-maupassant-le-horla` | Guy de Maupassant — *Le Horla* | FR | SHORT_WORK | 34 | **`PASS`** | Texte intégral, authentique, pagination stable, absence de boilerplate. |
| 21 | `fr-maupassant-boule-de-suif` | Guy de Maupassant — *Boule de Suif* | FR | SHORT_WORK | 55 | **`PASS`** | Texte intégral, authentique, pagination stable, absence de boilerplate. |
| 22 | `fr-maupassant-la-maison-tellier` | Guy de Maupassant — *La Maison Tellier* | FR | SHORT_WORK | 38 | **`PASS`** | Texte intégral, authentique, pagination stable, absence de boilerplate. |
| 23 | `fr-schwob-vies-imaginaires` | Marcel Schwob — *Vies imaginaires* | FR | LONG_FORM | 113 | **`PASS`** | Texte intégral, authentique, pagination stable, absence de boilerplate. |
| 24 | `fr-schwob-livre-de-monelle` | Marcel Schwob — *Le Livre de Monelle* | FR | SHORT_WORK | 75 | **`PASS`** | Texte intégral, authentique, pagination stable, absence de boilerplate. |
| 25 | `fr-schwob-croisade-des-enfants` | Marcel Schwob — *La Croisade des enfants* | FR | SHORT_WORK | 23 | **`PASS`** | Texte intégral, authentique, pagination stable, absence de boilerplate. |
| 26 | `fr-lesage-diable-boiteux` | Alain-René Lesage — *Le Diable boiteux* | FR | LONG_FORM | 384 | **`PASS`** | Texte intégral, authentique, pagination stable, absence de boilerplate. |
| 27 | `es-cervantes-quijote` | Miguel de Cervantes — *Don Quijote de la Mancha* | ES | LONG_FORM | 1365 | **`PASS`** | Texte intégral, authentique, pagination stable, absence de boilerplate. |
| 28 | `es-cervantes-rinconete-y-cortadillo` | Miguel de Cervantes — *Rinconete y Cortadillo* | ES | SHORT_WORK | 48 | **`PASS`** | Texte intégral, authentique, pagination stable, absence de boilerplate. |
| 29 | `es-cervantes-la-gitanilla` | Miguel de Cervantes — *La gitanilla* | ES | SHORT_WORK | 81 | **`PASS`** | Texte intégral, authentique, pagination stable, absence de boilerplate. |
| 30 | `es-cervantes-el-licenciado-vidriera` | Miguel de Cervantes — *El licenciado Vidriera* | ES | SHORT_WORK | 34 | **`PASS`** | Texte intégral, authentique, pagination stable, absence de boilerplate. |
| 31 | `es-cervantes-el-celoso-extremeno` | Miguel de Cervantes — *El celoso extremeño* | ES | SHORT_WORK | 49 | **`PASS`** | Texte intégral, authentique, pagination stable, absence de boilerplate. |
| 32 | `es-cervantes-el-coloquio-de-los-perros` | Miguel de Cervantes — *El coloquio de los perros* | ES | SHORT_WORK | 82 | **`PASS`** | Texte intégral, authentique, pagination stable, absence de boilerplate. |
| 33 | `es-quiroga-a-la-deriva` | Horacio Quiroga — *A la deriva* | ES | SHORT_WORK | 4 | **`PASS`** | Texte intégral, authentique, pagination stable, absence de boilerplate. |
| 34 | `es-quiroga-el-almohadon-de-plumas` | Horacio Quiroga — *El almohadón de plumas* | ES | SHORT_WORK | 5 | **`PASS`** | Texte intégral, authentique, pagination stable, absence de boilerplate. |
| 35 | `es-quiroga-la-insolacion` | Horacio Quiroga — *La insolación* | ES | SHORT_WORK | 9 | **`PASS`** | Texte intégral, authentique, pagination stable, absence de boilerplate. |
| 36 | `fr-merimee-mateo-falcone` | Prosper Mérimée — *Mateo Falcone* | FR | SHORT_WORK | 16 | **`PASS`** | Texte intégral, authentique, pagination stable, absence de boilerplate. |
| 37 | `fr-merimee-carmen` | Prosper Mérimée — *Carmen* | FR | SHORT_WORK | 81 | **`PASS`** | Texte intégral, authentique, pagination stable, absence de boilerplate. |
| 38 | `fr-daudet-chevre-seguin` | Alphonse Daudet — *La chèvre de monsieur Seguin* | FR | SHORT_WORK | 7 | **`PASS`** | Texte intégral, authentique, pagination stable, absence de boilerplate. |
| 39 | `fr-daudet-secret-cornille` | Alphonse Daudet — *Le secret de maître Cornille* | FR | SHORT_WORK | 8 | **`PASS`** | Texte intégral, authentique, pagination stable, absence de boilerplate. |
| 40 | `fr-daudet-trois-messes-basses` | Alphonse Daudet — *Les trois messes basses* | FR | SHORT_WORK | 11 | **`PASS`** | Texte intégral, authentique, pagination stable, absence de boilerplate. |
| 41 | `en-poe-tell-tale-heart` | Edgar Allan Poe — *The Tell-Tale Heart* | EN | SHORT_WORK | 7 | **`PASS`** | Texte intégral, authentique, pagination stable, absence de boilerplate. |
| 42 | `en-poe-cask-amontillado` | Edgar Allan Poe — *The Cask of Amontillado* | EN | SHORT_WORK | 8 | **`PASS`** | Texte intégral, authentique, pagination stable, absence de boilerplate. |
| 43 | `en-poe-oval-portrait` | Edgar Allan Poe — *The Oval Portrait* | EN | SHORT_WORK | 5 | **`PASS`** | Texte intégral, authentique, pagination stable, absence de boilerplate. |
| 44 | `en-doyle-scandal-bohemia` | Arthur Conan Doyle — *A Scandal in Bohemia* | EN | SHORT_WORK | 31 | **`PASS`** | Texte intégral, authentique, pagination stable, absence de boilerplate. |
| 45 | `en-doyle-red-headed-league` | Arthur Conan Doyle — *The Red-Headed League* | EN | SHORT_WORK | 33 | **`PASS`** | Texte intégral, authentique, pagination stable, absence de boilerplate. |
| 46 | `en-london-to-build-a-fire` | Jack London — *To Build a Fire* | EN | SHORT_WORK | 26 | **`PASS`** | Texte intégral, authentique, pagination stable, absence de boilerplate. |
| 47 | `en-henry-gift-of-the-magi` | O. Henry — *The Gift of the Magi* | EN | SHORT_WORK | 7 | **`PASS`** | Texte intégral, authentique, pagination stable, absence de boilerplate. |
| 48 | `en-henry-last-leaf` | O. Henry — *The Last Leaf* | EN | SHORT_WORK | 8 | **`PASS`** | Texte intégral, authentique, pagination stable, absence de boilerplate. |
| 49 | `en-saki-open-window` | Saki (H. H. Munro) — *The Open Window* | EN | SHORT_WORK | 4 | **`PASS`** | Texte intégral, authentique, pagination stable, absence de boilerplate. |
| 50 | `en-chopin-story-of-an-hour` | Kate Chopin — *The Story of an Hour* | EN | SHORT_WORK | 4 | **`PASS`** | Texte intégral, authentique, pagination stable, absence de boilerplate. |
| 51 | `en-lamb-essays-of-elia` | Charles Lamb — *Essays of Elia* | EN | LONG_FORM | 262 | **`PASS`** | Texte intégral, authentique, pagination stable, absence de boilerplate. |
| 52 | `en-wilde-dorian-gray` | Oscar Wilde — *The Picture of Dorian Gray* | EN | LONG_FORM | 278 | **`PASS`** | Texte intégral, authentique, pagination stable, absence de boilerplate. |
| 53 | `en-melville-bartleby` | Herman Melville — *Bartleby, the Scrivener* | EN | SHORT_WORK | 53 | **`PASS`** | Texte intégral, authentique, pagination stable, absence de boilerplate. |
| 54 | `fr-diderot-jacques-le-fataliste` | Denis Diderot — *Jacques le fataliste et son maître* | FR | LONG_FORM | 329 | **`PASS`** | Texte intégral, authentique, pagination stable, absence de boilerplate. |
| 55 | `fr-diderot-neveu-de-rameau` | Denis Diderot — *Le Neveu de Rameau* | FR | SHORT_WORK | 112 | **`PASS`** | Texte intégral, authentique, pagination stable, absence de boilerplate. |
| 56 | `fr-nerval-aurelia` | Gérard de Nerval — *Aurélia ou le Rêve et la Vie* | FR | SHORT_WORK | 86 | **`PASS`** | Texte intégral, authentique, pagination stable, absence de boilerplate. |
| 57 | `fr-barbey-les-diaboliques` | Jules Barbey d'Aurevilly — *Les Diaboliques* | FR | LONG_FORM | 371 | **`REVIEW`** | Comprend une préface d'auteur et avertissement éditeur avant le recueil des 6 nouvelles. |
| 58 | `fr-huysmans-a-rebours` | Joris-Karl Huysmans — *À rebours* | FR | LONG_FORM | 284 | **`PASS`** | Texte intégral, authentique, pagination stable, absence de boilerplate. |
| 59 | `fr-bloy-histoires-desobligeantes` | Léon Bloy — *Histoires désobligeantes* | FR | SHORT_WORK | 194 | **`PASS`** | Texte intégral, authentique, pagination stable, absence de boilerplate. |
| 60 | `fr-stendhal-chartreuse-de-parme` | Stendhal — *La Chartreuse de Parme* | FR | LONG_FORM | 684 | **`PASS`** | Texte intégral, authentique, pagination stable, absence de boilerplate. |
| 61 | `fr-stendhal-chroniques-italiennes` | Stendhal — *Chroniques italiennes* | FR | SHORT_WORK | 279 | **`PASS`** | Texte intégral, authentique, pagination stable, absence de boilerplate. |
| 62 | `es-quevedo-los-suenos` | Francisco de Quevedo — *Los sueños* | ES | LONG_FORM | 269 | **`PASS`** | Texte intégral, authentique, pagination stable, absence de boilerplate. |
| 63 | `es-quevedo-el-buscon` | Francisco de Quevedo — *Historia de la vida del Buscón* | ES | LONG_FORM | 152 | **`PASS`** | Texte intégral, authentique, pagination stable, absence de boilerplate. |
| 64 | `es-larra-el-doncel` | Mariano José de Larra — *El doncel de don Enrique el Doliente* | ES | LONG_FORM | 417 | **`PASS`** | Texte intégral, authentique, pagination stable, absence de boilerplate. |
| 65 | `es-clarin-adios-cordera` | Leopoldo Alas (Clarín) — *¡Adiós, Cordera! y otros cuentos* | ES | SHORT_WORK | 14 | **`PASS`** | Texte intégral, authentique, pagination stable, absence de boilerplate. |
| 66 | `es-pardo-bazan-los-pazos-de-ulloa` | Emilia Pardo Bazán — *Los pazos de Ulloa* | ES | LONG_FORM | 320 | **`PASS`** | Texte intégral, authentique, pagination stable, absence de boilerplate. |
| 67 | `es-galdos-misericordia` | Benito Pérez Galdós — *Misericordia* | ES | LONG_FORM | 304 | **`PASS`** | Texte intégral, authentique, pagination stable, absence de boilerplate. |
| 68 | `es-galdos-dona-perfecta` | Benito Pérez Galdós — *Doña Perfecta* | ES | LONG_FORM | 248 | **`PASS`** | Texte intégral, authentique, pagination stable, absence de boilerplate. |
| 69 | `es-unamuno-niebla` | Miguel de Unamuno — *Niebla* | ES | LONG_FORM | 202 | **`REVIEW`** | Structure méta-fictionnelle (nivola) avec prologue de Victor Goti et post-prologue d'Unamuno. |
| 70 | `es-unamuno-san-manuel-bueno-martir` | Miguel de Unamuno — *San Manuel Bueno, mártir* | ES | SHORT_WORK | 48 | **`PASS`** | Texte intégral, authentique, pagination stable, absence de boilerplate. |
| 71 | `es-valle-inclan-sonata-otono` | Ramón del Valle-Inclán — *Sonata de otoño (Memorias del Marqués de Bradomín)* | ES | SHORT_WORK | 76 | **`PASS`** | Texte intégral, authentique, pagination stable, absence de boilerplate. |
| 72 | `es-valle-inclan-sonata-primavera` | Ramón del Valle-Inclán — *Sonata de primavera* | ES | SHORT_WORK | 70 | **`PASS`** | Texte intégral, authentique, pagination stable, absence de boilerplate. |
| 73 | `es-valle-inclan-luces-de-bohemia` | Ramón del Valle-Inclán — *Luces de bohemia (esperpento)* | ES | SHORT_WORK | 71 | **`REVIEW`** | Format théâtral (esperpento) : didascalies continues et dialogues en prose expressive, à valider sur renderer mobile. |

---

## 3 — BILAN DES CRITÈRES QUALITATIFS (A À E)

L'inspection systématique automatisée et par échantillonnage des 73 œuvres normalisées et paginées établit :
- **A. DÉBUT :**
  - 100 % des incipits commencent directement sur le texte littéraire ou le titre de chapitre.
  - 0 licence Gutenberg résiduelle en tête.
  - 0 table de navigation parasite ou liens d'ancrage.
- **B. FIN :**
  - 0 licence, footer, clause de donation ou paratexte de fin.
  - 0 troncature résiduelle : `en-james-portrait-of-a-lady` réparé par concaténation des Volumes I & II (55 chapitres intégraux).
- **C. STRUCTURE :**
  - Titres et sous-titres hiérarchisés préservés (`heading` h1 à h6).
  - Conservation des styles inline (`italic`, `bold`) sans aplatissement.
  - Préservation des séparateurs de scènes (`scene_break`).
- **D. PAGINATION :**
  - **0 page vide** sur l'intégralité des 12,071 pages.
  - **0 duplication de page** (hashes SHA-256 strictement uniques par œuvre).
  - Continuité de séquence mathématique parfaite (`1, 2, ..., N`).
  - Longueur moyenne : 1569.8 caractères (médiane : 1602, P5 : 1231, P95 : 1772).
- **E. NETTOYAGE :**
  - Absence totale des motifs de marquage technique (`*** START OF`, `*** END OF`, `mw-parser-output`, etc.).

---

## 4 — BILAN DE LA RÉPARATION DES 13 ANCIENNES ŒUVRES « BLOCK »

Toutes les 13 œuvres précédemment bloquées ont été réparées avec succès selon des sources vérifiées, authentiques et rigoureusement délimitées. **Aucune œuvre ne demeure en statut BLOCK (0 BLOCK)**.

| Work ID | Auteur & Titre | Cause initiale | Source vérifiée / Remède | Pages avant | Pages après | Statut |
|---|---|---|---|---|---|---|
| `en-sterne-sentimental-journey` | Laurence Sterne — *A Sentimental Journey Through France and Italy* | `SOURCE_REPLACEMENT` | A Sentimental Journey through France and Italy | 500 | **141** | `REPAIRED_AND_VERIFIED` |
| `en-hazlitt-table-talk` | William Hazlitt — *Table-Talk* | `SOURCE_REPLACEMENT` | Table Talk: Essays on Men and Manners | 40 | **591** | `REPAIRED_AND_VERIFIED` |
| `en-gissing-henry-ryecroft` | George Gissing — *The Private Papers of Henry Ryecroft* | `SOURCE_REPLACEMENT` | The Private Papers of Henry Ryecroft | 88 | **230** | `REPAIRED_AND_VERIFIED` |
| `fr-mirbeau-journal-femme-chambre` | Octave Mirbeau — *Le Journal d'une femme de chambre* | `SOURCE_REPLACEMENT` | Le Journal d'une femme de chambre | 231 | **420** | `REPAIRED_AND_VERIFIED` |
| `fr-lesage-diable-boiteux` | Alain-René Lesage — *Le Diable boiteux* | `SOURCE_REPLACEMENT` | Le diable boiteux (Tomes I et II) | 100 | **384** | `REPAIRED_AND_VERIFIED` |
| `fr-merimee-carmen` | Prosper Mérimée — *Carmen* | `SOURCE_REPLACEMENT` | Carmen (texte original français) | 84 | **81** | `REPAIRED_AND_VERIFIED` |
| `es-larra-el-doncel` | Mariano José de Larra — *El doncel de don Enrique el Doliente* | `SOURCE_REPLACEMENT` | El doncel de don Enrique el doliente (Tomos I-IV) | 61 | **417** | `REPAIRED_AND_VERIFIED` |
| `es-valle-inclan-sonata-otono` | Ramón del Valle-Inclán — *Sonata de otoño (Memorias del Marqués de Bradomín)* | `SOURCE_REPLACEMENT` | Sonata de otoño; Sonata de invierno (PG #46182) | 28 | **76** | `REPAIRED_AND_VERIFIED` |
| `en-james-portrait-of-a-lady` | Henry James — *The Portrait of a Lady* | `MULTI_VOLUME_ASSEMBLY` | The Portrait of a Lady (Volumes I & II) | 424 | **833** | `REPAIRED_AND_VERIFIED` |
| `en-lamb-essays-of-elia` | Charles Lamb — *Essays of Elia* | `TEXT_EXTRACTION_AND_CLEANING` | The Works of Charles and Mary Lamb, Vol 2 (Elia 1823) | 728 | **262** | `REPAIRED_AND_VERIFIED` |
| `fr-maupassant-boule-de-suif` | Guy de Maupassant — *Boule de Suif* | `TEXT_EXTRACTION_AND_CLEANING` | Boule de Suif (Recueil Ollendorff 1899) | 141 | **55** | `REPAIRED_AND_VERIFIED` |
| `es-galdos-dona-perfecta` | Benito Pérez Galdós — *Doña Perfecta* | `TEXT_EXTRACTION_AND_CLEANING` | Doña Perfecta (PG #15725, excluding Notes & Vocabulary) | 424 | **248** | `REPAIRED_AND_VERIFIED` |
| `fr-nerval-aurelia` | Gérard de Nerval — *Aurélia ou le Rêve et la Vie* | `TEXT_EXTRACTION_AND_CLEANING` | Aurélia (Première et Seconde Parties, Wikisource) | 120 | **86** | `REPAIRED_AND_VERIFIED` |

---

## 5 — DÉTAIL DES 3 ŒUVRES EN « REVIEW »

1. **`fr-barbey-les-diaboliques`** (Jules Barbey d'Aurevilly — *Les Diaboliques*) :
   - **Spécificité :** Comprend une préface d'auteur et avertissement éditeur avant le recueil des 6 nouvelles.
   - **Action :** Validation humaine du confort de lecture sur écran mobile avant mise en production.

1. **`es-unamuno-niebla`** (Miguel de Unamuno — *Niebla*) :
   - **Spécificité :** Structure méta-fictionnelle (nivola) avec prologue de Victor Goti et post-prologue d'Unamuno.
   - **Action :** Validation humaine du confort de lecture sur écran mobile avant mise en production.

1. **`es-valle-inclan-luces-de-bohemia`** (Ramón del Valle-Inclán — *Luces de bohemia (esperpento)*) :
   - **Spécificité :** Format théâtral (esperpento) : didascalies continues et dialogues en prose expressive, à valider sur renderer mobile.
   - **Action :** Validation humaine du confort de lecture sur écran mobile avant mise en production.

---

## 6 — PRIORITÉS TYPOGRAPHIQUES POUR LE RENDU MOBILE (BANC DE TEST IPHONE)

Les œuvres ci-dessous présentent des caractéristiques formelles riches ou complexes. Elles constituent le banc de test prioritaire pour le composant React Native `ReadingContent` et les transitions de lecture :

| Work ID | Auteur & Titre | Langue | Caractéristiques typographiques clés | Statut Qualité |
|---|---|---|---|---|
| `en-austen-pride-prejudice` | Jane Austen — *Pride and Prejudice* | EN | Italiques intenses (468), Dialogues tiret cadratin (—), Séparateurs de scène (75) | `PASS` |
| `en-austen-persuasion` | Jane Austen — *Persuasion* | EN | Dialogues tiret cadratin (—), Séparateurs de scène (1) | `PASS` |
| `en-johnson-rasselas` | Samuel Johnson — *The History of Rasselas, Prince of Abissinia* | EN | Dialogues tiret cadratin (—), Forme épistolaire | `PASS` |
| `en-sterne-sentimental-journey` | Laurence Sterne — *A Sentimental Journey Through France and Italy* | EN | Italiques intenses (332), Dialogues tiret cadratin (—) | `PASS` |
| `en-browne-religio-medici` | Sir Thomas Browne — *Religio Medici* | EN | Italiques intenses (130), Dialogues tiret cadratin (—) | `PASS` |
| `en-browne-hydriotaphia` | Sir Thomas Browne — *Hydriotaphia, Urn Burial* | EN | Dialogues tiret cadratin (—) | `PASS` |
| `en-hazlitt-table-talk` | William Hazlitt — *Table-Talk* | EN | Italiques intenses (613), Vers / poésie (126 strophes), Dialogues tiret cadratin (—), Séparateurs de scène (3), Forme épistolaire | `PASS` |
| `en-gissing-henry-ryecroft` | George Gissing — *The Private Papers of Henry Ryecroft* | EN | Dialogues tiret cadratin (—) | `PASS` |
| `en-james-portrait-of-a-lady` | Henry James — *The Portrait of a Lady* | EN | Italiques intenses (283), Dialogues tiret cadratin (—), Séparateurs de scène (5) | `PASS` |
| `en-james-turn-of-the-screw` | Henry James — *The Turn of the Screw* | EN | Italiques intenses (225), Dialogues tiret cadratin (—), Séparateurs de scène (1) | `PASS` |
| `en-james-daisy-miller` | Henry James — *Daisy Miller* | EN | Dialogues tiret cadratin (—), Séparateurs de scène (2), Forme épistolaire | `PASS` |
| `en-james-beast-in-the-jungle` | Henry James — *The Beast in the Jungle* | EN | Dialogues tiret cadratin (—) | `PASS` |
| `en-james-aspern-papers` | Henry James — *The Aspern Papers* | EN | Dialogues tiret cadratin (—), Séparateurs de scène (2) | `PASS` |
| `fr-mirbeau-journal-femme-chambre` | Octave Mirbeau — *Le Journal d'une femme de chambre* | FR | Dialogues tiret cadratin (—) | `PASS` |
| `fr-maupassant-bel-ami` | Guy de Maupassant — *Bel-Ami* | FR | Italiques intenses (103), Dialogues tiret cadratin (—), Séparateurs de scène (1) | `PASS` |
| `fr-maupassant-pierre-et-jean` | Guy de Maupassant — *Pierre et Jean* | FR | Dialogues tiret cadratin (—), Séparateurs de scène (3) | `PASS` |
| `fr-maupassant-la-parure` | Guy de Maupassant — *La Parure* | FR | Dialogues tiret cadratin (—) | `PASS` |
| `fr-maupassant-la-ficelle` | Guy de Maupassant — *La Ficelle* | FR | Dialogues tiret cadratin (—) | `PASS` |
| `fr-maupassant-deux-amis` | Guy de Maupassant — *Deux amis* | FR | Dialogues tiret cadratin (—) | `PASS` |
| `fr-maupassant-le-horla` | Guy de Maupassant — *Le Horla* | FR | Dialogues tiret cadratin (—), Séparateurs de scène (1) | `PASS` |
| `fr-maupassant-boule-de-suif` | Guy de Maupassant — *Boule de Suif* | FR | Dialogues tiret cadratin (—), Séparateurs de scène (6) | `PASS` |
| `fr-maupassant-la-maison-tellier` | Guy de Maupassant — *La Maison Tellier* | FR | Dialogues tiret cadratin (—) | `PASS` |
| `fr-schwob-vies-imaginaires` | Marcel Schwob — *Vies imaginaires* | FR | Dialogues tiret cadratin (—), Séparateurs de scène (1) | `PASS` |
| `fr-schwob-livre-de-monelle` | Marcel Schwob — *Le Livre de Monelle* | FR | Dialogues tiret cadratin (—) | `PASS` |
| `fr-schwob-croisade-des-enfants` | Marcel Schwob — *La Croisade des enfants* | FR | Dialogues tiret cadratin (—) | `PASS` |
| `fr-lesage-diable-boiteux` | Alain-René Lesage — *Le Diable boiteux* | FR | Italiques intenses (189), Dialogues tiret cadratin (—), Séparateurs de scène (21), Forme épistolaire | `PASS` |
| `es-cervantes-quijote` | Miguel de Cervantes — *Don Quijote de la Mancha* | ES | Dialogues tiret cadratin (—), Ponctuation inversée (¿, ¡), Séparateurs de scène (1), Théâtre / didascalies | `PASS` |
| `es-cervantes-rinconete-y-cortadillo` | Miguel de Cervantes — *Rinconete y Cortadillo* | ES | Dialogues tiret cadratin (—), Ponctuation inversée (¿, ¡), Séparateurs de scène (2) | `PASS` |
| `es-cervantes-la-gitanilla` | Miguel de Cervantes — *La gitanilla* | ES | Dialogues tiret cadratin (—), Ponctuation inversée (¿, ¡), Séparateurs de scène (2) | `PASS` |
| `es-cervantes-el-licenciado-vidriera` | Miguel de Cervantes — *El licenciado Vidriera* | ES | Dialogues tiret cadratin (—), Ponctuation inversée (¿, ¡), Séparateurs de scène (2) | `PASS` |
| `es-cervantes-el-celoso-extremeno` | Miguel de Cervantes — *El celoso extremeño* | ES | Dialogues tiret cadratin (—), Ponctuation inversée (¿, ¡), Séparateurs de scène (2) | `PASS` |
| `es-cervantes-el-coloquio-de-los-perros` | Miguel de Cervantes — *El coloquio de los perros* | ES | Dialogues tiret cadratin (—), Ponctuation inversée (¿, ¡), Séparateurs de scène (1), Théâtre / didascalies | `PASS` |
| `es-quiroga-a-la-deriva` | Horacio Quiroga — *A la deriva* | ES | Dialogues tiret cadratin (—), Ponctuation inversée (¿, ¡) | `PASS` |
| `es-quiroga-el-almohadon-de-plumas` | Horacio Quiroga — *El almohadón de plumas* | ES | Dialogues tiret cadratin (—), Ponctuation inversée (¿, ¡) | `PASS` |
| `es-quiroga-la-insolacion` | Horacio Quiroga — *La insolación* | ES | Dialogues tiret cadratin (—), Ponctuation inversée (¿, ¡) | `PASS` |
| `fr-merimee-mateo-falcone` | Prosper Mérimée — *Mateo Falcone* | FR | Dialogues tiret cadratin (—), Séparateurs de scène (1) | `PASS` |
| `fr-merimee-carmen` | Prosper Mérimée — *Carmen* | FR | Italiques intenses (131), Dialogues tiret cadratin (—) | `PASS` |
| `fr-daudet-chevre-seguin` | Alphonse Daudet — *La chèvre de monsieur Seguin* | FR | Dialogues tiret cadratin (—), Séparateurs de scène (6) | `PASS` |
| `fr-daudet-secret-cornille` | Alphonse Daudet — *Le secret de maître Cornille* | FR | Dialogues tiret cadratin (—), Séparateurs de scène (2) | `PASS` |
| `fr-daudet-trois-messes-basses` | Alphonse Daudet — *Les trois messes basses* | FR | Dialogues tiret cadratin (—) | `PASS` |
| `en-poe-tell-tale-heart` | Edgar Allan Poe — *The Tell-Tale Heart* | EN | Dialogues tiret cadratin (—) | `PASS` |
| `en-poe-cask-amontillado` | Edgar Allan Poe — *The Cask of Amontillado* | EN | Dialogues tiret cadratin (—) | `PASS` |
| `en-poe-oval-portrait` | Edgar Allan Poe — *The Oval Portrait* | EN | Dialogues tiret cadratin (—) | `PASS` |
| `en-doyle-scandal-bohemia` | Arthur Conan Doyle — *A Scandal in Bohemia* | EN | Dialogues tiret cadratin (—) | `PASS` |
| `en-doyle-red-headed-league` | Arthur Conan Doyle — *The Red-Headed League* | EN | Dialogues tiret cadratin (—) | `PASS` |
| `en-london-to-build-a-fire` | Jack London — *To Build a Fire* | EN | Dialogues tiret cadratin (—) | `PASS` |
| `en-henry-gift-of-the-magi` | O. Henry — *The Gift of the Magi* | EN | Dialogues tiret cadratin (—), Forme épistolaire | `PASS` |
| `en-henry-last-leaf` | O. Henry — *The Last Leaf* | EN | Dialogues tiret cadratin (—) | `PASS` |
| `en-saki-open-window` | Saki (H. H. Munro) — *The Open Window* | EN | Forme épistolaire | `PASS` |
| `en-chopin-story-of-an-hour` | Kate Chopin — *The Story of an Hour* | EN | Dialogues tiret cadratin (—), Séparateurs de scène (1) | `PASS` |
| `en-lamb-essays-of-elia` | Charles Lamb — *Essays of Elia* | EN | Italiques intenses (402), Dialogues tiret cadratin (—) | `PASS` |
| `en-wilde-dorian-gray` | Oscar Wilde — *The Picture of Dorian Gray* | EN | Dialogues tiret cadratin (—), Séparateurs de scène (1) | `PASS` |
| `en-melville-bartleby` | Herman Melville — *Bartleby, the Scrivener* | EN | Dialogues tiret cadratin (—), Séparateurs de scène (2) | `PASS` |
| `fr-diderot-jacques-le-fataliste` | Denis Diderot — *Jacques le fataliste et son maître* | FR | Italiques intenses (163), Dialogues tiret cadratin (—), Séparateurs de scène (2) | `PASS` |
| `fr-diderot-neveu-de-rameau` | Denis Diderot — *Le Neveu de Rameau* | FR | Dialogues tiret cadratin (—) | `PASS` |
| `fr-nerval-aurelia` | Gérard de Nerval — *Aurélia ou le Rêve et la Vie* | FR | Dialogues tiret cadratin (—), Séparateurs de scène (5) | `PASS` |
| `fr-barbey-les-diaboliques` | Jules Barbey d'Aurevilly — *Les Diaboliques* | FR | Dialogues tiret cadratin (—) | `REVIEW` |
| `fr-huysmans-a-rebours` | Joris-Karl Huysmans — *À rebours* | FR | Italiques intenses (168), Dialogues tiret cadratin (—), Séparateurs de scène (1), Forme épistolaire | `PASS` |
| `fr-bloy-histoires-desobligeantes` | Léon Bloy — *Histoires désobligeantes* | FR | Italiques intenses (237), Dialogues tiret cadratin (—) | `PASS` |
| `fr-stendhal-chartreuse-de-parme` | Stendhal — *La Chartreuse de Parme* | FR | Dialogues tiret cadratin (—), Séparateurs de scène (1) | `PASS` |
| `fr-stendhal-chroniques-italiennes` | Stendhal — *Chroniques italiennes* | FR | Italiques intenses (114), Vers / poésie (3 strophes), Dialogues tiret cadratin (—), Séparateurs de scène (6) | `PASS` |
| `es-quevedo-los-suenos` | Francisco de Quevedo — *Los sueños* | ES | Italiques intenses (1821), Dialogues tiret cadratin (—), Ponctuation inversée (¿, ¡), Séparateurs de scène (7) | `PASS` |
| `es-quevedo-el-buscon` | Francisco de Quevedo — *Historia de la vida del Buscón* | ES | Ponctuation inversée (¿, ¡) | `PASS` |
| `es-larra-el-doncel` | Mariano José de Larra — *El doncel de don Enrique el Doliente* | ES | Italiques intenses (148), Dialogues tiret cadratin (—), Ponctuation inversée (¿, ¡), Séparateurs de scène (191), Théâtre / didascalies | `PASS` |
| `es-clarin-adios-cordera` | Leopoldo Alas (Clarín) — *¡Adiós, Cordera! y otros cuentos* | ES | Dialogues tiret cadratin (—), Ponctuation inversée (¿, ¡), Séparateurs de scène (2), Théâtre / didascalies, Forme épistolaire | `PASS` |
| `es-pardo-bazan-los-pazos-de-ulloa` | Emilia Pardo Bazán — *Los pazos de Ulloa* | ES | Italiques intenses (235), Dialogues tiret cadratin (—), Ponctuation inversée (¿, ¡), Séparateurs de scène (34), Théâtre / didascalies | `PASS` |
| `es-galdos-misericordia` | Benito Pérez Galdós — *Misericordia* | ES | Italiques intenses (1078), Dialogues tiret cadratin (—), Ponctuation inversée (¿, ¡), Séparateurs de scène (43), Théâtre / didascalies | `PASS` |
| `es-galdos-dona-perfecta` | Benito Pérez Galdós — *Doña Perfecta* | ES | Italiques intenses (121), Dialogues tiret cadratin (—), Ponctuation inversée (¿, ¡), Théâtre / didascalies | `PASS` |
| `es-unamuno-niebla` | Miguel de Unamuno — *Niebla* | ES | Dialogues tiret cadratin (—), Ponctuation inversée (¿, ¡), Séparateurs de scène (43), Théâtre / didascalies | `REVIEW` |
| `es-unamuno-san-manuel-bueno-martir` | Miguel de Unamuno — *San Manuel Bueno, mártir* | ES | Dialogues tiret cadratin (—), Ponctuation inversée (¿, ¡) | `PASS` |
| `es-valle-inclan-sonata-otono` | Ramón del Valle-Inclán — *Sonata de otoño (Memorias del Marqués de Bradomín)* | ES | Dialogues tiret cadratin (—), Ponctuation inversée (¿, ¡), Séparateurs de scène (1), Forme épistolaire | `PASS` |
| `es-valle-inclan-sonata-primavera` | Ramón del Valle-Inclán — *Sonata de primavera* | ES | Ponctuation inversée (¿, ¡) | `PASS` |
| `es-valle-inclan-luces-de-bohemia` | Ramón del Valle-Inclán — *Luces de bohemia (esperpento)* | ES | Dialogues tiret cadratin (—), Ponctuation inversée (¿, ¡), Séparateurs de scène (27), Théâtre / didascalies | `REVIEW` |

---

## 7 — STATISTIQUES RÉELLES DE PAGINATION DU CORPUS V2

- **Total pages réelles :** `12,071`
- **Pages vides :** `0`
- **Pages dupliquées :** `0`
- **Caractères par page :** Moyenne = `1569.8`, Médiane = `1602`, Min = `109`, Max = `1849`
- **Centiles de distribution :** P5 = `1231`, P95 = `1772`
- **Répartition par langue :**
  - **EN** : `4,018` pages (33.3 %) sur `26` œuvres
  - **FR** : `4,185` pages (34.7 %) sur `26` œuvres
  - **ES** : `3,868` pages (32.0 %) sur `21` œuvres

---

## 8 — RÈGLE D'ARRÊT STRICTE

- Aucun import Railway n'a été exécuté.
- Aucun artefact de source n'a été altéré.
- L'édition `curated_v1` est scellée par son verrou machine `curated_v1.lock.json`.
- Ce rapport a été produit mécaniquement par sérialisation de l'état réel du disque.
