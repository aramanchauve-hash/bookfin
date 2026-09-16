# Pride and Prejudice — literal line-break audit

Status: diagnostic only. No Curated V1 content, hash, pagination, or status was changed.

## Evidence

The Gutenberg source keeps editorially meaningless source wrapping inside HTML
paragraphs. For example, `sources/en/en-austen-pride-prejudice.html` lines
366–367 place a newline between “possession” and “of a good fortune” inside a
single `<p>`.

The same literal `\r\n` is present in the normalized document at
`normalized/en/en-austen-pride-prejudice.json` line 38, then unchanged in the
paginated output at `pages/en/en-austen-pride-prejudice_pages.json` line 46.

## Attribution

| Stage | Finding |
| --- | --- |
| Source | HTML source uses physical line wrapping inside `<p>` elements. |
| Normalizer | Preserves those literal wraps instead of normalizing prose whitespace. |
| Paginator | Copies the normalized blocks; it does not introduce the defect. |
| Renderer | Correctly receives V2 text with the literal wraps; it must not repair corpus semantics. |

## Likely related works

The same `\r\n` signature occurs in 18 English page files, including
`en-wilde-dorian-gray`, `en-austen-persuasion`, `en-james-turn-of-the-screw`,
`en-hazlitt-table-talk`, and several Doyle/James works. They should be audited
by a future normalizer-only correction with a new corpus validation process.
