# Curated V1 — Railway import plan

**Mode: preparation only.** This report does not authorize an import, migration,
deployment, deletion, or Railway configuration change.

## Current Railway state

Read-only Railway status identified `bookfin-api` as online in production
(region `sfo`), with an online PostgreSQL service. The observed deployment ID
is `45d1a720-b662-45b5-bc81-1dcf55186a66`; public `/health` returned HTTP 200.
There is no application-version endpoint, so this deployment ID is the only
deployed-version evidence.

Detailed SQL inventory did not run: the private PostgreSQL hostname injected
by Railway cannot be resolved from this workstation (`host unknown`). No SQL
statement ran, no secret was printed, and no Railway resource was modified.
Therefore migrations, live corpus counts, and live `content_v2` remain
unverified pending the tunnel preflight below.

From source, the feed uses active `pages`, `user_language_preferences`, and
`page_impressions`; `works` and `editions` supply metadata. `books`,
`extracts`, and `extract_impressions` are legacy/backfill tables. Back up all
tables, including reactions, profiles, affinities, and language preferences.

## Prepared safeguards

The existing `ingest_curated_v1` remains the sole importer. Its new
`--apply-railway` mode requires a Railway hostname (never localhost, 127.0.0.1,
or Neon), `BOOKFIN_RAILWAY_TARGET=production`,
`BOOKFIN_RAILWAY_IMPORT_CONFIRMATION=replace-alpha-with-curated-v1`, and an
exact `BOOKFIN_EXPECTED_ACTIVE_ALPHA_PAGES` preflight count.

The atomic transaction imports Curated V1 then deactivates only active V1
pages. It never deletes pages, users, impressions, reactions, or profiles. It
rolls back if the alpha count changes or the only active rows are not exactly
the 12,025 Curated V2 pages. The prepared Railway build packages the importer
and read-only audit/verifier binaries, while still starting only `bookfin`.
The prepared API source no longer logs a full `DATABASE_URL`.

## Mandatory preflight and backup

Install PostgreSQL client tools (`pg_dump`, `pg_restore`, `psql`). In terminal
A, create an SSH-only tunnel and leave it running:

```powershell
railway connect Postgres --environment production --ssh --tunnel-only --port 55432
```

In terminal B, put the URI printed by that command into a process-local
variable only; never echo or commit it. Then audit the database read-only:

```powershell
$env:BOOKFIN_TUNNEL_DATABASE_URL = '<URI printed by railway connect>'
$env:DATABASE_URL = $env:BOOKFIN_TUNNEL_DATABASE_URL
$env:BOOKFIN_RAILWAY_AUDIT = 'bookfin-curated-v1-readonly-audit'
$env:BOOKFIN_RAILWAY_TUNNEL_CONFIRMATION = 'postgres-production-ssh-tunnel'
cargo run --bin audit_railway_readonly
```

Required results: migrations 1 through 11, `pages.content_v2_present=true`,
edition inventory, table counts, and the exact active V1 page count. Record
that count as `<ALPHA_PAGE_COUNT>`. Stop if migration/schema differs, alpha is
not exclusively active V1, or any unknown active V2 corpus is present.

Create and validate a full, timestamped backup outside the repository:

```powershell
$backupStamp = Get-Date -Format 'yyyyMMdd-HHmmss'
$backupDir = Join-Path $env:USERPROFILE 'Bookfin-Railway-Backups'
New-Item -ItemType Directory -Force -Path $backupDir | Out-Null
$backupFile = Join-Path $backupDir "bookfin-pre-curated-v1-$backupStamp.dump"
pg_dump --format=custom --file=$backupFile --dbname=$env:BOOKFIN_TUNNEL_DATABASE_URL
Get-FileHash -Algorithm SHA256 $backupFile
pg_restore --list $backupFile
```

The custom dump covers the entire database. Restore-rehearse it only into a
new isolated database, never production:

```powershell
createdb bookfin_restore_rehearsal
pg_restore --clean --if-exists --dbname=bookfin_restore_rehearsal $backupFile
dropdb bookfin_restore_rehearsal
```

## Future authorized execution sequence

These commands require the later explicit instruction “Exécute l'import
Railway.” Do not run them in this pass.

```powershell
git status --short
git rev-parse HEAD
python scripts/test_curated_v1_integrity.py
python scripts/test_corpus_v2.py
cargo run --bin ingest_curated_v1 --
```

After the preflight/backup above, deploy the reviewed prepared commit. Normal
API startup applies forward migrations, including 0011:

```powershell
railway up --service bookfin-api --environment production
railway status
```

Audit the now-deployed service read-only, then run the guarded importer inside
the service, where Railway private DNS resolves. Replace the placeholder only
with the recorded integer:

```powershell
railway ssh --service bookfin-api --environment production -- sh -lc 'BOOKFIN_RAILWAY_AUDIT=bookfin-curated-v1-readonly-audit ./target/release/audit_railway_readonly'
railway ssh --service bookfin-api --environment production -- sh -lc 'BOOKFIN_RAILWAY_TARGET=production BOOKFIN_RAILWAY_IMPORT_CONFIRMATION=replace-alpha-with-curated-v1 BOOKFIN_EXPECTED_ACTIVE_ALPHA_PAGES=<ALPHA_PAGE_COUNT> ./target/release/ingest_curated_v1 --apply-railway'
railway ssh --service bookfin-api --environment production -- sh -lc 'BOOKFIN_RAILWAY_TARGET=production BOOKFIN_RAILWAY_VERIFY_CONFIRMATION=verify-curated-v1-production ./target/release/verify_curated_v1_railway'
```

The final verifier proves the 73 deterministic manifest work IDs, 12,025
pages, EN 26 / FR 26 / ES 21 editions, non-null V2 payloads, no empty page,
continuous sequences, no duplicate coordinates, and no active non-Curated
page. Deterministic UUIDs are used because source work slugs are not database
columns. The pre-import integrity test proves no HOLD, PROPOSED, or REJECTED
work is in the manifest.

Perform `curl.exe -i https://bookfin-api-production.up.railway.app/health`
first. Feed FR/EN/ES, V2-block/hash, strict exhaustion, LIKE, DISLIKE, and
CONTINUE_READING require disposable test identities and write data, so they
are intentionally excluded from this preparation pass. Mobile smoke follows
only after API acceptance.

## SQL post-import acceptance checks

```sql
SELECT count(DISTINCT w.id) AS curated_works, count(p.id) AS curated_pages
FROM works w JOIN editions e ON e.work_id = w.id JOIN pages p ON p.edition_id = e.id
WHERE e.source_name = 'Bookfin Curated V1';

SELECT language_tag, count(DISTINCT edition_id)
FROM pages p JOIN editions e ON e.id = p.edition_id
WHERE e.source_name = 'Bookfin Curated V1'
GROUP BY language_tag ORDER BY language_tag;

SELECT count(*) AS null_v2_or_empty
FROM pages p JOIN editions e ON e.id = p.edition_id
WHERE e.source_name = 'Bookfin Curated V1' AND (p.content_v2 IS NULL OR btrim(p.content) = '');

SELECT count(*) AS active_non_curated
FROM pages p JOIN editions e ON e.id = p.edition_id
WHERE p.is_active = true AND e.source_name <> 'Bookfin Curated V1';
```

Expected: 73 / 12,025; EN 26 / FR 26 / ES 21; and zero for both final checks.
The verifier additionally checks exact accepted work identities, hashes,
sequences, and duplicates.

## Rollback, Git, and risks

Before commit, the importer rolls back itself. After commit, stop the API only
if needed, restore the verified full dump through an approved maintenance path,
redeploy the pre-import commit, then recheck health and an alpha feed. Never
manually delete Curated rows or rewrite served page content.

The Bookfin worktree is currently dirty: tracked mobile/backend files are
modified and Curated V1 corpus, migrations, scripts, reports, and binaries are
untracked. Review the exact `git status --short` output and create one reviewed
commit before production; do not use `git add .` blindly. Other repositories
are irrelevant.

Hard stops: successful tunnel audit, readable backup plus restore rehearsal,
reviewed/pushed commit, and explicit human authorization. Estimated future
window: 20–35 minutes, excluding backup transfer/restore. The non-return point
is importer transaction commit; the validated backup is the rollback path.

## Execution log — 2026-09-16

**AUDIT SQL TUNNEL: PASS**
`railway ssh keys add` registered a newly generated local key (none existed
before), then `railway connect Postgres --environment production --ssh
--tunnel-only --port 55432` opened cleanly. `audit_railway_readonly` ran
through it and printed no secret.

**Audit result — state mismatch, import path blocked:**
Railway production is at `migrations=[1..9]` only; migrations 0010 and 0011
have never been applied, and `pages.content_v2_present=false`. The live
corpus is the *old* alpha set — 52 works, 52 editions, **15,570** pages
(EN 15,417 / ES 18 / FR 135; all version 1, all active) — not the 73-work /
12,025-page Curated V1 set this plan assumed. Per this document's own rule
("if real state differs from what the importer expects: STOP"), no schema
change, migration, or import was attempted against Railway. This is the
blocking finding: `BOOKFIN_EXPECTED_ACTIVE_ALPHA_PAGES` cannot yet be set
correctly, and `ingest_curated_v1 --apply-railway` should not be run until a
human reconciles which corpus is actually supposed to be live.

**BACKUP CREATED: PASS**
Full `pg_dump --format=custom` of the tunneled database, stored outside the
repo at `~/Bookfin-Railway-Backups/bookfin-pre-curated-v1-20260916-184148.dump`
(11.7 MB, SHA-256 `17722ccf133880a64a68fc552b048b3821f98e81c1a81401544bbf2bf923620a`).
Not committed; contents not exposed.

**BACKUP INSPECTED: PASS**
`pg_restore --list` confirmed CUSTOM format, 117 TOC entries, all 17 Bookfin
tables present (schema + data + the `prevent_page_content_mutation` trigger
function).

**RESTORE TEST 1: PASS**
Restored into throwaway local DB `bookfin_railway_restore_test`: all 17
tables present, counts matched the Railway audit exactly (works=52,
editions=52, pages=15570, users=2, reactions=7), migrations 1–9 present, zero
invalid FK constraints. The `bookfin` API started against this database and
`/health` returned 200 locally.

**RESTORE TEST 2: PASS**
Repeated into a second, separately created throwaway DB
(`bookfin_railway_restore_test2`); identical counts and migration count
confirmed reproducibility. Both throwaway databases were dropped afterward.

**GIT REVIEW: PASS**
Reviewed `git status`/`diff` in full. No secrets found (only the pre-existing
local dev placeholder `postgres://bookfin:bookfin@localhost...` and a test
constant literally named to never be printed). No backup file is inside the
repo. One real issue was found and corrected before commit:
`mobile/src/lib/reader/bookfinFont.ts` had two conflicting top-level
declarations of `poliphiliFontSource`/`poliphiliIsInstalled` — a broken
merge that also happened to hide a licensing question, since the font
directory's own README gates those files on a confirmed embedding licence.
Licensing was confirmed by the user; the duplicate declaration was removed
and the single licensed code path kept.

**TESTS BEFORE COMMIT: PASS**
`cargo check --bins`, `cargo test` (0 failures across 45 backend tests),
`python scripts/test_curated_v1_integrity.py`,
`scripts/test_corpus_v2.py`, `scripts/test_spanish_light_normalization.py`,
`scripts/test_spanish_source_audit.py`, `scripts/test_hazlitt_table_talk_audit.py`
all passed. Mobile `npm run typecheck` and `npm test` (14 suites / 97 tests)
passed after the `bookfinFont.ts` fix. EAS was not invoked.

**COMMIT: `0e1179a0f779f303850699db81a3c26d865dc2bb`** — "Prepare Curated V1
Railway import"

**PUSH: NOT DONE — no git remote is configured for this repository**
(`git remote -v` is empty). Nothing was pushed; no remote-triggered Railway
deployment could have occurred as a result of this pass.

**POST-PUSH /health: PASS (N/A — no push occurred)**
`curl -i https://bookfin-api-production.up.railway.app/health` returned 200
at the end of this pass, unrelated to any push since none happened.

**PRODUCTION CORPUS STILL UNCHANGED: YES**
Only SELECT statements and `pg_dump` (read-only) ran against Railway
Postgres. No import, migration, or write of any kind was executed there.

READY TO EXECUTE RAILWAY IMPORT: NO

Two things block it, independent of each other: (1) the corpus/migration
mismatch above must be reconciled by a human — is the 15,570-page alpha
corpus actually what should be live, or does Railway need migrations 10/11
and a real Curated V1 import; and (2) there is no git remote, so "reviewed
and pushed" cannot be completed until one is configured and the local commit
is pushed to it.

## Superseded — 2026-09-19

Both blockers above were resolved and the cutover was executed and verified.
See `docs/reports/CURATED_V1_PRODUCTION_CUTOVER_PLAN.md` §12 for the full
execution log: a GitHub remote was created, migrations 10/11 were applied,
and Curated V1 (73 works / 12,025 pages) replaced the alpha corpus as the
live, served content. This document is kept for its pre-cutover analysis;
it no longer reflects Railway's current state.

READY TO EXECUTE RAILWAY IMPORT: YES (executed 2026-09-19)
