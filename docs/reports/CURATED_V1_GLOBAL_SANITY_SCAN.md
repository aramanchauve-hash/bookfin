# Curated V1 — Global Sanity Scan

Read-only scan of source HTML → normalized V2 JSON → paginated V2 JSON. No file under `corpus/` is written by this command.

## Result

- Works: **73**; paginated pages: **12025**
- PASS: **48**; REVIEW: **25**; BLOCK: **0**
- BLOCK is reserved for a missing/unreadable canonical artifact or a failed pagination invariant; a regex candidate alone cannot block release.
- Source-only false positives filtered by V2: **244**

## Top REVIEW candidates

- `es-quevedo-los-suenos`: GLUED_CASE×6, PAGE_MARKER×50
- `fr-lesage-diable-boiteux`: HTML_NAV×2, PAGE_MARKER×32, TOC×2
- `en-browne-religio-medici`: GLUED_CASE×13
- `fr-maupassant-bel-ami`: GLUED_CASE×8, SOFT_HYPHEN×2, TOC×2
- `en-hazlitt-table-talk`: FACSIMILE×2, PAGE_MARKER×6

## Per-work classification

| Work | Lang | Status | Anomalies / invariant | Probable layer | Signature count | Source-only FP |
| --- | --- | --- | --- | --- | ---: | ---: |
| `en-austen-pride-prejudice` | en | **PASS** | — | NONE | 0 | 4 |
| `en-austen-persuasion` | en | **PASS** | — | NONE | 0 | 4 |
| `en-johnson-rasselas` | en | **REVIEW** | PAGE_MARKER×6 | SOURCE/NORMALIZER | 6 | 5 |
| `en-sterne-sentimental-journey` | en | **REVIEW** | GLUED_CASE×2 | SOURCE/NORMALIZER | 2 | 4 |
| `en-browne-religio-medici` | en | **REVIEW** | GLUED_CASE×13 | SOURCE/NORMALIZER | 13 | 5 |
| `en-browne-hydriotaphia` | en | **PASS** | — | NONE | 0 | 5 |
| `en-hazlitt-table-talk` | en | **REVIEW** | FACSIMILE×2, PAGE_MARKER×6 | SOURCE/NORMALIZER | 8 | 5 |
| `en-gissing-henry-ryecroft` | en | **PASS** | — | NONE | 0 | 4 |
| `en-james-portrait-of-a-lady` | en | **PASS** | — | NONE | 0 | 3 |
| `en-james-turn-of-the-screw` | en | **PASS** | — | NONE | 0 | 3 |
| `en-james-daisy-miller` | en | **PASS** | — | NONE | 0 | 3 |
| `en-james-beast-in-the-jungle` | en | **PASS** | — | NONE | 0 | 3 |
| `en-james-aspern-papers` | en | **PASS** | — | NONE | 0 | 3 |
| `fr-mirbeau-journal-femme-chambre` | fr | **PASS** | — | NONE | 0 | 3 |
| `fr-maupassant-bel-ami` | fr | **REVIEW** | GLUED_CASE×8, SOFT_HYPHEN×2, TOC×2 | SOURCE/NORMALIZER | 12 | 3 |
| `fr-maupassant-pierre-et-jean` | fr | **PASS** | — | NONE | 0 | 3 |
| `fr-maupassant-la-parure` | fr | **PASS** | — | NONE | 0 | 3 |
| `fr-maupassant-la-ficelle` | fr | **PASS** | — | NONE | 0 | 3 |
| `fr-maupassant-deux-amis` | fr | **PASS** | — | NONE | 0 | 3 |
| `fr-maupassant-le-horla` | fr | **PASS** | — | NONE | 0 | 3 |
| `fr-maupassant-boule-de-suif` | fr | **PASS** | — | NONE | 0 | 1 |
| `fr-maupassant-la-maison-tellier` | fr | **PASS** | — | NONE | 0 | 3 |
| `fr-schwob-vies-imaginaires` | fr | **REVIEW** | GLUED_CASE×2 | SOURCE/NORMALIZER | 2 | 3 |
| `fr-schwob-livre-de-monelle` | fr | **PASS** | — | NONE | 0 | 3 |
| `fr-schwob-croisade-des-enfants` | fr | **PASS** | — | NONE | 0 | 3 |
| `fr-lesage-diable-boiteux` | fr | **REVIEW** | HTML_NAV×2, PAGE_MARKER×32, TOC×2 | SOURCE/NORMALIZER | 36 | 5 |
| `es-cervantes-quijote` | es | **PASS** | — | NONE | 0 | 3 |
| `es-cervantes-rinconete-y-cortadillo` | es | **PASS** | — | NONE | 0 | 4 |
| `es-cervantes-la-gitanilla` | es | **PASS** | — | NONE | 0 | 4 |
| `es-cervantes-el-licenciado-vidriera` | es | **PASS** | — | NONE | 0 | 4 |
| `es-cervantes-el-celoso-extremeno` | es | **PASS** | — | NONE | 0 | 4 |
| `es-cervantes-el-coloquio-de-los-perros` | es | **PASS** | — | NONE | 0 | 4 |
| `es-quiroga-a-la-deriva` | es | **PASS** | — | NONE | 0 | 3 |
| `es-quiroga-el-almohadon-de-plumas` | es | **PASS** | — | NONE | 0 | 3 |
| `es-quiroga-la-insolacion` | es | **PASS** | — | NONE | 0 | 3 |
| `fr-merimee-mateo-falcone` | fr | **PASS** | — | NONE | 0 | 3 |
| `fr-merimee-carmen` | fr | **REVIEW** | PAGE_MARKER×2 | SOURCE/NORMALIZER | 2 | 4 |
| `fr-daudet-chevre-seguin` | fr | **PASS** | — | NONE | 0 | 3 |
| `fr-daudet-secret-cornille` | fr | **PASS** | — | NONE | 0 | 3 |
| `fr-daudet-trois-messes-basses` | fr | **PASS** | — | NONE | 0 | 3 |
| `en-poe-tell-tale-heart` | en | **PASS** | — | NONE | 0 | 3 |
| `en-poe-cask-amontillado` | en | **REVIEW** | GLUED_CASE×2 | SOURCE/NORMALIZER | 2 | 3 |
| `en-poe-oval-portrait` | en | **REVIEW** | GLUED_CASE×2 | SOURCE/NORMALIZER | 2 | 3 |
| `en-doyle-scandal-bohemia` | en | **PASS** | — | NONE | 0 | 4 |
| `en-doyle-red-headed-league` | en | **PASS** | — | NONE | 0 | 4 |
| `en-london-to-build-a-fire` | en | **REVIEW** | GLUED_CASE×2 | SOURCE/NORMALIZER | 2 | 3 |
| `en-henry-gift-of-the-magi` | en | **PASS** | — | NONE | 0 | 3 |
| `en-henry-last-leaf` | en | **PASS** | — | NONE | 0 | 3 |
| `en-saki-open-window` | en | **PASS** | — | NONE | 0 | 3 |
| `en-chopin-story-of-an-hour` | en | **REVIEW** | GLUED_CASE×2 | SOURCE/NORMALIZER | 2 | 3 |
| `en-lamb-essays-of-elia` | en | **PASS** | — | NONE | 0 | 2 |
| `en-wilde-dorian-gray` | en | **PASS** | — | NONE | 0 | 3 |
| `en-melville-bartleby` | en | **PASS** | — | NONE | 0 | 3 |
| `fr-diderot-jacques-le-fataliste` | fr | **REVIEW** | GLUED_CASE×2, PAGE_MARKER×6 | SOURCE/NORMALIZER | 8 | 4 |
| `fr-diderot-neveu-de-rameau` | fr | **REVIEW** | GLUED_CASE×2 | SOURCE/NORMALIZER | 2 | 3 |
| `fr-nerval-aurelia` | fr | **REVIEW** | FACSIMILE×2, GLUED_CASE×2 | SOURCE/NORMALIZER | 4 | 4 |
| `fr-barbey-les-diaboliques` | fr | **REVIEW** | GUTENBERG×2, TOC×2 | SOURCE/NORMALIZER | 4 | 3 |
| `fr-huysmans-a-rebours` | fr | **REVIEW** | GLUED_CASE×2 | SOURCE/NORMALIZER | 2 | 3 |
| `fr-bloy-histoires-desobligeantes` | fr | **REVIEW** | GLUED_CASE×6 | SOURCE/NORMALIZER | 6 | 3 |
| `fr-stendhal-chartreuse-de-parme` | fr | **REVIEW** | GLUED_CASE×2 | SOURCE/NORMALIZER | 2 | 3 |
| `fr-stendhal-chroniques-italiennes` | fr | **REVIEW** | GLUED_CASE×2, HTML_TAG×4 | SOURCE/NORMALIZER | 6 | 3 |
| `es-quevedo-los-suenos` | es | **REVIEW** | GLUED_CASE×6, PAGE_MARKER×50 | SOURCE/NORMALIZER | 56 | 5 |
| `es-quevedo-el-buscon` | es | **PASS** | — | NONE | 0 | 3 |
| `es-larra-el-doncel` | es | **REVIEW** | GUTENBERG×8 | SOURCE/NORMALIZER | 8 | 5 |
| `es-clarin-adios-cordera` | es | **PASS** | — | NONE | 0 | 3 |
| `es-pardo-bazan-los-pazos-de-ulloa` | es | **PASS** | — | NONE | 0 | 3 |
| `es-galdos-misericordia` | es | **PASS** | — | NONE | 0 | 3 |
| `es-galdos-dona-perfecta` | es | **PASS** | — | NONE | 0 | 1 |
| `es-unamuno-niebla` | es | **REVIEW** | PAGE_MARKER×4 | SOURCE/NORMALIZER | 4 | 5 |
| `es-unamuno-san-manuel-bueno-martir` | es | **PASS** | — | NONE | 0 | 3 |
| `es-valle-inclan-sonata-otono` | es | **REVIEW** | HTML_TAG×2 | SOURCE/NORMALIZER | 2 | 2 |
| `es-valle-inclan-sonata-primavera` | es | **REVIEW** | GLUED_CASE×2 | SOURCE/NORMALIZER | 2 | 3 |
| `es-valle-inclan-luces-de-bohemia` | es | **PASS** | — | NONE | 0 | 4 |

## REVIEW evidence

- `en-johnson-rasselas` — PAGE_MARKER, NORMALIZER: “p. 93“We have hitherto,” said she, “known but little of the worl”
- `en-sterne-sentimental-journey` — GLUED_CASE, NORMALIZER: “LePOURet”
- `en-browne-religio-medici` — GLUED_CASE, NORMALIZER: “lineWhich”
- `en-hazlitt-table-talk` — FACSIMILE, NORMALIZER: “Burleigh was an exact and wonderful facsimile of nature, and I resolved to make mine (as nearly as I co”
- `fr-maupassant-bel-ami` — SOFT_HYPHEN, NORMALIZER: “Comment se présen­terait-elle ? Il n’en savait rien, mais il l’attendait depui”
- `fr-schwob-vies-imaginaires` — GLUED_CASE, NORMALIZER: “MeurRenatus”
- `fr-lesage-diable-boiteux` — PAGE_MARKER, NORMALIZER: “et une vénération méritée (T. II, p. 146), c'est le Lieutenant de police d'Argenson.”
- `fr-merimee-carmen` — PAGE_MARKER, NORMALIZER: “nt. (Voir Anales de Sevilla, t. II, p. 136). Quoi qu’il en soit, il existe encore à Séville une rue du”
- `en-poe-cask-amontillado` — GLUED_CASE, NORMALIZER: “domainPublic”
- `en-poe-oval-portrait` — GLUED_CASE, NORMALIZER: “domainPublic”
- `en-london-to-build-a-fire` — GLUED_CASE, NORMALIZER: “domainPublic”
- `en-chopin-story-of-an-hour` — GLUED_CASE, NORMALIZER: “domainPublic”
- `fr-diderot-jacques-le-fataliste` — PAGE_MARKER, NORMALIZER: “Musset-Pathay, Paris, 1821, t. II, p.320, une partie des renseignements que nous avons à donner sur”
- `fr-diderot-neveu-de-rameau` — GLUED_CASE, NORMALIZER: “eReader”
- `fr-nerval-aurelia` — GLUED_CASE, NORMALIZER: “auréliaLachenal”
- `fr-barbey-les-diaboliques` — TOC, NORMALIZER: “Table des matières”
- `fr-huysmans-a-rebours` — GLUED_CASE, NORMALIZER: “deNotre”
- `fr-bloy-histoires-desobligeantes` — GLUED_CASE, NORMALIZER: “lePaindans”
- `fr-stendhal-chartreuse-de-parme` — GLUED_CASE, NORMALIZER: “CroyezA”
- `fr-stendhal-chroniques-italiennes` — GLUED_CASE, NORMALIZER: “deValmontone”
- `es-quevedo-los-suenos` — PAGE_MARKER, NORMALIZER: “. (Tribunal de esta justa venganza, pág. 37).”
- `es-larra-el-doncel` — GUTENBERG, NORMALIZER: “g/details/eldonceldedonenr01larr Project Gutenberg has the other three volumes of this work. Volume II: see ht”
- `es-unamuno-niebla` — PAGE_MARKER, NORMALIZER: “e libro, sea novela o nivola (véase pág. 158)—y conste que esto de la nivola es invención mía—, no pocos”
- `es-valle-inclan-sonata-otono` — HTML_TAG, NORMALIZER: “<img alt="OPERA OMNIA </body>”
- `es-valle-inclan-sonata-primavera` — GLUED_CASE, NORMALIZER: “contemPlando”

## BLOCK evidence

None.

## Signature totals (canonical V2 only)

- `FACSIMILE`: 4
- `GLUED_CASE`: 59
- `GUTENBERG`: 10
- `HTML_NAV`: 2
- `HTML_TAG`: 6
- `PAGE_MARKER`: 106
- `SOFT_HYPHEN`: 2
- `TOC`: 6
