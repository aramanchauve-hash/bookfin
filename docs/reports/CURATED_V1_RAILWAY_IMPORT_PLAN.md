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

READY TO EXECUTE RAILWAY IMPORT: NO
