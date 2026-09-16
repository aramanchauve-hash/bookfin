# Curated V1 — Schema and importer dry-run audit

## Scope

The initial preflight was local and static: no database URL was read, no
database was opened, and no Railway resource was contacted. The later local
PostgreSQL execution is recorded in its own dated section below.

The reproducible dry run is:

```powershell
python scripts/dry_run_import_curated_v1.py
```

It verifies the manifest, all 73 source/normalized/pages triples, matching
document hashes, continuous pages, non-empty V2 blocks and 64-character V2
content hashes. It emits a deterministic plan under `target/` only:
`target/curated_v1_import_dry_run.json`.

Result: **73 works, 12,025 pages, zero database access**.

## Compatibility finding

| Concern | Current state | Import consequence |
| --- | --- | --- |
| `works` / `editions` | Tables and provenance columns exist (migrations 0003, 0009) | Compatible for a dedicated Curated V1 edition. |
| Plain page text | `pages.content TEXT` exists | Compatible only as legacy/search fallback. |
| V2 blocks | Migration `0011_page_content_v2.sql` adds nullable `pages.content_v2 JSONB`; Rust uses explicit `PageContentV2` / `BlockV2` / `SpanV2`; feed DTO exposes optional `blocks` | Compatible. Legacy alpha rows stay nullable; Curated V1 importer always supplies V2 blocks. |
| Canonical V2 page hash | Migration `0011` adds `canonical_content_hash` alongside legacy `content_hash` | Compatible. The importer retains the canonical source hash and keeps legacy flattened text only as a fallback. |
| Served-page immutability | Migration 0006 protects content of served pages | Future importer must create a dedicated edition/version; it must not update served pages. |
| Migration runner | `run_migrations` previously deleted migration-ledger rows ≥4 | Corrected locally: migrations now run normally without deleting `_sqlx_migrations`. |

## Required implementation before any production import

1. Apply forward-only migration `0011_page_content_v2.sql`; it does not alter
   old pages or migration-ledger history.
2. Use `cargo run --bin ingest_curated_v1 --` for a filesystem-only Rust
   round-trip dry run (73 works / 12,025 pages). The importer uses typed V2
   blocks, deterministic UUIDv5 identities and never repaginates.
3. Run `cargo run --bin ingest_curated_v1 -- --apply-local` only against an
   isolated localhost database; the binary rejects non-local database URLs.
4. Compare exact counts and sample API responses, then obtain explicit
   authorization before any Railway operation.

## Idempotence model

The local dry run derives identities with UUIDv5 from a fixed namespace and
`kind/work_id/page_sequence/version`. Re-running it produces the same row IDs,
canonical V2 hashes and block JSON hashes. It records both the canonical V2
hash and a legacy flattened-text hash so a future importer cannot confuse them.

## Implemented V2 path and validation

`0011_page_content_v2.sql` adds nullable `content_v2 JSONB` and
`canonical_content_hash` with shape/hash checks. `PageContentV2`, `BlockV2`
and `SpanV2` are serde types covering paragraph, heading, scene break,
blockquote, verse, italic, bold and small caps. Repository reads deserialize
JSONB into the domain page; `FeedPageDto.blocks` returns the exact structured
blocks directly to the already-compatible mobile type. `text` remains only a
legacy fallback.

`cargo run --bin ingest_curated_v1 --` passed locally: 73 works / 12,025
pages parsed and V2-deserialized/re-serialized with no database connection.
The unit round-trip additionally covers FR/EN/ES characters, heading, scene
break, blockquote, verse, italic, bold and small caps.

## Strict random exhaustion

The SQL repository already excludes `page_impressions` in both primary and
wrap-around selections. The failing integration test was sharing the global
French pool with concurrently inserted test pages. It now uses a dedicated
language preference and pages, proving the actual strict contract: an unseen
page is returned once, seen pages are excluded, and the exhausted eligible
pool returns `None`. The HTTP endpoint serializes that state as HTTP 404 with
`{"error":"feed_exhausted"}`; the mobile `ApiError.isFeedExhausted` helper
distinguishes it from a transport failure.

## Historical validation boundary (superseded)

No isolated temporary PostgreSQL provisioning tool is installed in this
workspace (`psql`, `createdb` and Docker are unavailable). The existing local
`bookfin` database is intentionally not mutated. Therefore the database apply
and API smoke phases remain pending an isolated localhost PostgreSQL database;
Railway remains untouched.

## Final isolated PostgreSQL attempt (2026-09-15)

A dedicated, empty Neon project named `bookfin-curated-v1-dry-run` was created
in `aws-eu-central-1`; it was not linked to the workspace and its URL was
injected only into child processes. The pre-existing Neon project named
`spontaneous-travel` was not used.

The importer first refused the remote URL by design. It was then given a
separate `--apply-isolated` mode which requires both a Neon hostname and the
exact `BOOKFIN_ISOLATED_DB_PURPOSE=bookfin-curated-v1-dry-run` marker; Railway
and ordinary remote URLs remain rejected. SQLx then failed before migrations
could run with Windows socket error `OS 10060` (remote compute did not respond
to the direct PostgreSQL TCP connection). No migration, table write, Curated
import, SQL check, API smoke test or idempotence run therefore completed.

The temporary Neon project was deleted immediately after the failed attempt.
It had no user data and no application data was imported. The remaining
unverified steps require an environment with outbound direct PostgreSQL access
to an explicitly isolated database.

**READY FOR RAILWAY IMPORT PREPARATION: NO** — the only remaining blocker is
the real PostgreSQL/API validation, blocked by direct TCP connectivity; it is
not a corpus, migration, Rust, mobile, or random-feed test failure.

## Local isolated PostgreSQL validation (2026-09-15)

This supersedes the preceding remote-connectivity boundary. The test used the
already-running local PostgreSQL 18.4 instance on `localhost:5432`, never a
Railway, Neon, or other remote URL. The historical Bookfin `.env` configuration
identified the pre-existing `bookfin` database and `bookfin` role. It contained
11 applied migrations, 540 works and 17,864 pages, and was inspected only.

An exact, separate database named `bookfin_curated_v1_dry_run` was then
created on that same instance. The importer was invoked with `--apply-local`;
that mode accepts only an exact localhost/127.0.0.1 URL whose database name is
`bookfin_curated_v1_dry_run`. All migrations, including 0011, applied there.

| Check | Result |
| --- | --- |
| Curated import | PASS — 73 works / 12,025 pages |
| `content_v2 IS NULL` | PASS — 0 |
| sequence continuity / duplicate page identities | PASS — 0 breaks / 0 duplicates |
| language counts | PASS — EN 26, FR 26, ES 21 |
| V2 JSONB structure in imported pages | PASS — 1,135 heading and 115 verse blocks; typed round-trip tests also cover blockquote and italic spans |
| second import (idempotence) | PASS — unchanged counts and no duplicates |
| local API `/health` | PASS — HTTP 200 |
| local API feeds | PASS — FR, EN and ES each returned a matching language page with non-empty V2 `blocks` and `content_hash` |
| strict exhaustion | PASS — HTTP 404 `{"error":"feed_exhausted"}` for an isolated `ru` preference with no eligible page |

The local API was stopped after the test. The exact temporary database was
removed successfully; no other local database was targeted.

**READY FOR RAILWAY IMPORT PREPARATION: YES** — this is a validation verdict
only. No Railway migration, query, import, or deployment was performed.

## Explicit non-actions

- No Railway migration, connection, seed, query or deployment was run.
- No EAS command was run.
- No corpus content, source hash, status or review decision was modified.
