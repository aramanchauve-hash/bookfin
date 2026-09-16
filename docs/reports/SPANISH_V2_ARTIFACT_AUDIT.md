# Spanish Curated V1 artifact audit

Diagnostic only: no corpus content, hashes, pages, or statuses were changed.

| Work | Source evidence | Normalized → paginated evidence | Responsible layer |
| --- | --- | --- | --- |
| `es-cervantes-quijote` | Source contains a table of contents made of `div.toc`, headings, and list items. | The normalized first block concatenates headings and contents (`letrasDonde`); the paginated page copies it. | Normalizer block-boundary / whitespace handling. |
| `es-unamuno-niebla` | Source line 641 has inline `<span class="pagenum">[Pg 13]</span>`. | `[Pg 13]` survives normalized JSON and paginated pages. | Normalizer must discard `pagenum` spans. |
| `es-valle-inclan-luces-de-bohemia` | Source line 516 is `<p class="rol"><span class="pagenum">p. 14</span>MAX</p>`. | Normalized JSON and pages contain `p. 14MAX`. | Normalizer must discard `pagenum` before extracting speaker text. |

## Scan of the 21 Spanish page files

The generic signatures occur beyond the three fixtures:

- `[Pg N]`: `es-unamuno-niebla` (295), `es-quevedo-los-suenos` (301).
- `p. N`: all six listed Cervantes short works, `es-cervantes-quijote`,
  `es-larra-el-doncel`, `es-quevedo-los-suenos`, and
  `es-valle-inclan-luces-de-bohemia` (260).
- lower-case letter immediately followed by `Donde`: `es-cervantes-quijote`
  (45 occurrences), plus one in `es-quevedo-los-suenos`.

These are normalizer/source-mapping defects. They must be corrected in a
separate corpus rebuild and integrity-validation workflow, not hidden in CSS
or the mobile renderer.
