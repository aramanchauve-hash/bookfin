# Curated V1 — Pre-production freeze candidate

## Corpus identity

- Manifest: `Bookfin Curated Library V1`
- Manifest version: `1.2-spanish-light-normalization`
- Manifest SHA-256: `bd9c3e749efd4d499b5c6db876fb565ece4a7c09f0ce21b59a46377ec3b71528`
- Source tree revision inspected: `ad1343c7f05b2e9351f6cd10c058e2a597cf3e72`
- Works: 73 (EN 25, FR 26, ES 22)
- Canonical paginated pages: 12,025 (EN 4,018; FR 4,185; ES 3,822)
- Normalizer: `bookfin.document.v1`; paginator: `bookfin.page.v1`, version `2.0`

## Read-only quality gate

The global scan in [CURATED_V1_GLOBAL_SANITY_SCAN.md](CURATED_V1_GLOBAL_SANITY_SCAN.md) completed with:

- PASS: 48
- REVIEW: 25
- BLOCK: 0

The REVIEW rows are retained as evidence for editorial follow-up. They are not
automatic corpus mutations and no regex result is treated as a release block.
All 73 source/normalized/paginated triples are present and the page sequence,
non-empty page and intra-work content-hash invariants passed.

## Freeze decision

The corpus files are a **freeze candidate**. They must not be regenerated or
edited during the database-import preparation. The candidate is not production
ready yet because the existing backend schema/API cannot preserve V2 blocks;
the required, non-destructive schema/API work is recorded in
[CURATED_V1_SCHEMA_IMPORT_DRY_RUN.md](CURATED_V1_SCHEMA_IMPORT_DRY_RUN.md).
