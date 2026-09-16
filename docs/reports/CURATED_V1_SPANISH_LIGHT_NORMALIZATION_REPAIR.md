# Curated V1 — réparation espagnole « Keep + Light Normalization »

Date : 2026-09-14  
Portée : uniquement les dix œuvres espagnoles validées par
`CURATED_V1_SPANISH_SOURCE_AUDIT.md`.

## Résultat

Les sources HTML sont toutes inchangées. Seuls les documents JSON, pages et
échantillons des dix cibles ont été régénérés. Le normalizer passe à `2.1.0`;
le paginator reste `2.0` et n’a reçu aucune règle nouvelle.

| Œuvre | Règle DOM appliquée | Avant | Après |
| --- | --- | --- | --- |
| `es-cervantes-quijote` | Exclusion de `class="toc"` et de l’`ul` frère explicitement rattaché à son titre TOC. | 137 collages, dont `letrasDonde`; TOC dans le premier paragraphe. | 0 collage suspect ; aucune TOC canonique. |
| `es-cervantes-rinconete-y-cortadillo` | Ignorer `span.pagenum` dans la tranche Gutenberg explicitement définie. | 23 marqueurs `p. N`. | 0. |
| `es-cervantes-la-gitanilla` | Même règle `span.pagenum`, même tranche explicite. | 13 marqueurs `p. N`. | 0. |
| `es-cervantes-el-licenciado-vidriera` | Même règle `span.pagenum`, même tranche explicite. | 4 marqueurs `p. N`. | 0. |
| `es-cervantes-el-celoso-extremeno` | Même règle `span.pagenum`, même tranche explicite. | 6 marqueurs `p. N`. | 0. |
| `es-cervantes-el-coloquio-de-los-perros` | Même règle `span.pagenum`, même tranche explicite. | 18 marqueurs `p. N`. | 0. |
| `es-quevedo-los-suenos` | Ignorer `span.pagenum` et la table explicitement marquée `data-summary="cont"`. | 322 marqueurs de pagination ; table des matières concaténée. | 0 marqueur de pagination HTML ; table retirée. |
| `es-larra-el-doncel` | Ignorer `span.pagenum`, wrappers Gutenberg et blocs « The Project Gutenberg eBook ». | 627 marqueurs et 4 headers documentaires. | 0 / 0. |
| `es-unamuno-niebla` | Ignorer `span.pagenum`. | 297 marqueurs, dont `[Pg 13]`. | 0 marqueur HTML. |
| `es-valle-inclan-luces-de-bohemia` | Ignorer `span.pagenum`; préserver les frontières DOM `li` et `div.verse`. | 216 marqueurs, `p. 14MAX`, 5 collages. | 0, `MAX`, 0 collage. |

Total des nœuds source `span.pagenum` exclus avant création du texte canonique :
**1 698**. Les compteurs « avant » du tableau correspondent aux artefacts
visibles dans le JSON antérieur ; ils peuvent être inférieurs au compte DOM,
car l’ancien parser éliminait déjà certains nœuds au gré de ses frontières de
bloc. Les suppressions proviennent toutes de nœuds HTML balisés ; aucun mot
littéraire n’a été corrigé par expression régulière.

## Signatures restantes, justifiées

Le scanner lexical reste volontairement prudent et signale trois familles qui
ne sont pas des `span.pagenum` :

- *Los sueños* conserve 21 références éditoriales (`p. 388`, `[57]`, etc.) et
  trois collages déjà présents dans les notes critiques. Elles ne sont pas des
  marques de pagination HTML ; les enlever demanderait une décision éditoriale
  hors périmètre.
- *Niebla* conserve une phrase de note de transcription : « En la p. 268… ».
  Elle n’est pas le faux `[Pg 13]` et n’est donc pas supprimée.
- Le scanner ne trouve ni TOC, ni header/footer Gutenberg, ni concaténation
  suspecte dans les pages réparées de *Quijote*, *El doncel* ou *Luces*.

## Garde-fous et régénération

`scripts/repair_spanish_light_normalization.py` impose un snapshot préalable,
`corpus/curated_v1/spanish_light_normalization_guard.json`. Avant et après la
régénération, il vérifie :

- les 73 HTML source byte-identical ;
- les JSON, pages et échantillons des 63 œuvres hors cible byte-identical ;
- les mêmes 73 identifiants `ACCEPTED` dans le manifest et le lock.

Les cinq nouvelles de Cervantès continuent de lire leur tranche explicite dans
le master Gutenberg #61202 ; le script refuse de normaliser l’anthologie
entière à la place d’une œuvre.

Fichiers régénérés :

- `normalized/es/<10 work_id>.json`
- `pages/es/<10 work_id>_pages.json`
- `samples/es/<10 work_id>_samples.json`
- `manifest.json`, `curated_v1.lock.json`
- Fixtures mobile ciblées : `mobile_preview_fixture.json`,
  `niebla_pages.json`, `luces_de_bohemia_pages.json`,
  `don_quijote_pages.json`, `el_doncel_pages.json`.

Le manifest est maintenant `1.2-spanish-light-normalization` : **73 œuvres**,
**12 025 pages** dont **3 822 espagnoles**. Le lock contient le même set
`accepted_work_ids`, son total est `12 025`, et sa référence de normalizer est
`2.1.0`.

## Tests exécutés

```powershell
python scripts/test_spanish_source_audit.py
python scripts/test_spanish_light_normalization.py
python scripts/test_curated_v1_integrity.py
python scripts/test_corpus_v2.py
cd mobile
npm run typecheck
npm test -- --runInBand
```

Résultat : tous les tests passent (7 tests d’intégrité Curated V1, 8 tests
Corpus V2, 91 tests Jest mobile). Aucune opération Railway ou EAS n’a été
effectuée.

## Validation iPhone

```powershell
cd C:\Users\arama\Dev\Bookfin\mobile
$env:EXPO_PUBLIC_API_BASE_URL="https://bookfin-api-production.up.railway.app"
npx expo start -c
```

Dans le laboratoire DEV, vérifier *Don Quijote*, *El doncel*, *Niebla* et
*Luces de bohemia*. Les données viennent exclusivement des fixtures locales
synchronisées ; la production continue d’utiliser l’API.
