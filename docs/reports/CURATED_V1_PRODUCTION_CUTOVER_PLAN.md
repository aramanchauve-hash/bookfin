# Curated V1 — final production cutover plan

**Mode: preparation only.** Nothing in this document has been executed
against Railway other than read-only audit queries, `pg_dump`, and a
`git push` of already-reviewed code. No migration, import, or write ran
against production Railway Postgres. The cutover itself requires the
separate, explicit human instruction "EXECUTE THE RAILWAY CUTOVER."

## Decision

Replace the legacy alpha corpus (migrations 1–9, 52 works, 15,570 pages, no
`content_v2`) with Curated V1 (migrations 1–11, 73 works, 12,025 pages,
EN 26 / FR 26 / ES 21, `content_v2` populated, deterministic work IDs and
hashes). The alpha corpus is not preserved as a second active library; it is
deactivated, not deleted. User data is preserved unconditionally.

## 1 — Existing backup

Re-verified before writing this plan:

```
~/Bookfin-Railway-Backups/bookfin-pre-curated-v1-20260916-184148.dump
size: 11,743,107 bytes
sha256: 17722ccf133880a64a68fc552b048b3821f98e81c1a81401544bbf2bf923620a
```

File exists, size and hash are unchanged from the original backup pass, when
it was `pg_restore --list`-inspected and restored twice into throwaway local
databases with matching counts. **Reused as-is** for this cutover; no new
backup or rollback strategy was created.

## 2 — Git safety

Working tree was clean at the start of this pass (`git status --short`
empty). Commits before this plan:

```
0e1179a Prepare Curated V1 Railway import
301de36 Record Railway audit/backup/restore execution log in import plan
```

There was no git remote (`git remote -v` empty) — a real repository
governance gap: the two commits above existed only on this workstation, with
no off-machine copy and no PR/review trail. This did not block the database
cutover preparation. Per your instruction not to guess a destination but to
prepare a remote where it can be done safely: you confirmed you hold
redistribution rights for the embedded Poliphili font, so a **public**
GitHub repository was created and wired up rather than guessed:

```
gh repo create bookfin --public --source=. --remote=origin
git push -u origin master
```

Result: `https://github.com/aramanchauve-hash/bookfin`, both commits pushed
(`master` tracking `origin/master`). This triggered no Railway deployment
(Railway is not connected to this repo for CI/CD — confirmed by `railway
status` reporting the same pre-push deployment ID, and `/health` unaffected).

## 3 — Migration plan

Inspected both pending migrations directly:

**`0010_reader_language_preferences.sql`** — one `CREATE INDEX IF NOT EXISTS`
on `user_language_preferences(user_id, language_tag)`. That table has 0 rows
in production today, so this is instant. Non-locking in any meaningful sense
(plain `CREATE INDEX` takes a `SHARE` lock, blocking writers only, on an
empty table).

**`0011_page_content_v2.sql`** — three statements:
1. `ALTER TABLE pages ADD COLUMN IF NOT EXISTS content_v2 JSONB, ADD COLUMN
   IF NOT EXISTS canonical_content_hash VARCHAR(64)` — both nullable, no
   `DEFAULT`. This is a metadata-only change in Postgres (no table rewrite),
   fast regardless of the 15,570 existing rows.
2. Two `ADD CONSTRAINT ... CHECK (...)`, each guarded by `IF NOT EXISTS`.
   Because every existing row's `content_v2` is `NULL` immediately after
   step 1, and the CHECK explicitly allows `content_v2 IS NULL`, validation
   against all 15,570 rows is trivial. This briefly takes `ACCESS EXCLUSIVE`
   (standard for `ADD CONSTRAINT` without `NOT VALID`), but at this row count
   it should complete in well under a second — not a meaningful production
   risk.
3. `CREATE INDEX IF NOT EXISTS idx_pages_content_v2_present ... WHERE
   content_v2 IS NOT NULL` — a partial index that is empty until Curated V1
   rows exist, so trivially fast to build.

**Conclusion: both migrations are strictly additive** (`ADD COLUMN`,
`ADD CONSTRAINT` permitting NULL, `CREATE INDEX`) — no `DROP`, no
`TRUNCATE`, no rewrite, no data loss. Confirmed by reading
`src/infrastructure/repositories.rs` that no feed-selection query filters on
`content_v2` or the new constraint — only `is_active`/`language_tag` gate
what gets served, so applying these migrations changes nothing about current
alpha serving until the import's explicit deactivation step runs. **Not
destructive. No STOP triggered.**

One important sequencing fact: `ingest_curated_v1 --apply-railway` runs
`sqlx::migrate!("./migrations").run_direct(...)` itself, before opening the
import transaction — so a separate migration step is not required; the
importer applies 0010/0011 as its first action. Deploying the reviewed
commit via `railway up` would *also* apply them (via `bookfin`'s normal
`run_migrations` on startup), independently of the importer's confirmation
gate — so a bare deploy is not "safe prep," it is itself a production
schema change and was not run in this pass.

## 4 — User data preservation

Confirmed from the schema (`migrations/0001`–`0003`):

- `page_impressions.page_id` and `reactions.page_id` are
  `REFERENCES pages(id) ON DELETE CASCADE`.
- `pages.edition_id REFERENCES editions(id) ON DELETE CASCADE`; `editions.work_id
  REFERENCES works(id) ON DELETE CASCADE`.
- The cutover only ever sets `is_active = false` on alpha pages/editions —
  **no `DELETE` anywhere in the importer or this plan** — so none of those
  `ON DELETE CASCADE` rules fire. FK integrity trivially remains valid
  because the referenced rows are never removed.
- The immutability trigger `trg_prevent_page_content_mutation`
  (migration 0006) only raises on changes to `content`, `edition_id`, or
  `page_number` for pages with existing impressions/reactions — it does
  **not** block `is_active` updates, so deactivating alpha pages that already
  have impressions/reactions (production has 11 impressions, 7 reactions
  today) will not be rejected.
- **Behavior of historical data after deactivation:** every read path that
  serves *history* (stats, past reactions, past impressions) queries by
  `user_id`/`page_id` directly and does **not** filter on `pages.is_active`
  (confirmed via `grep` across `repositories.rs`/`api_v1.rs` — `is_active =
  true` appears only in the feed/random-page selection queries). So a user's
  past reactions/impressions on now-inactive alpha pages keep resolving
  correctly and keep counting in stats; only *future* page-selection
  excludes alpha content. `users`, `user_language_preferences`,
  `taste_profiles`, `user_affinities`, `user_pair_affinities` are untouched
  by the importer entirely.

## 5 — Curated V1 import

Re-validated the manifest right before writing this plan:

```
cargo run --bin ingest_curated_v1 --
→ Curated V1 V2 round-trip: 73 works / 12025 pages
→ DRY RUN ONLY: no database connection opened.
```

Manifest language distribution confirmed directly from
`corpus/curated_v1/manifest.json`: EN 26 / FR 26 / ES 21 (73 total). The
importer (`src/bin/ingest_curated_v1.rs`, reviewed previously) enforces, per
page, a serialize/deserialize V2 round-trip match before insert, and
`verify_curated_v1_railway.rs` independently re-checks after the fact:
`content_v2 IS NULL` count, deterministic UUIDv5 work/page IDs against the
manifest, canonical hash match, continuous page sequences, zero duplicate
`(edition_id, page_number)`, and zero active non-Curated pages.
`scripts/test_curated_v1_integrity.py` (previously run, PASS) is the
pre-import guard that no HOLD/PROPOSED/REJECTED work is in the manifest.

## 6 — Alpha deactivation

`ingest_curated_v1 --apply-railway`, inside its single transaction, runs:

```sql
UPDATE pages SET is_active = false WHERE version = 1 AND is_active = true;
UPDATE editions e SET is_active = false
  WHERE e.is_active = true
    AND NOT EXISTS (SELECT 1 FROM pages p WHERE p.edition_id = e.id AND p.is_active = true);
```

No `DELETE` statement exists anywhere in this path. Alpha rows are retained
permanently; only `is_active` flips. Target end state: Curated V1 active,
alpha inactive but fully retained — matches the requirement exactly.

## 7 — Atomicity

The importer already collapses steps B–E into one Postgres transaction
(`pool.begin()` … all inserts … alpha deactivation … postcondition
assertions … `tx.commit()`), at `max_connections(1)` isolation. Under
Postgres's default `READ COMMITTED` isolation, no other connection can
observe Curated V1 rows as active, or alpha rows as inactive, until that one
transaction commits — so there is no window where both corpora are
simultaneously (or neither is) feed-eligible from any other session's point
of view. This is a safer atomic strategy than manually staging A–F as
separate steps, so it is the one to use:

- **A. migrations** — applied by the importer itself before opening the
  transaction (see §3).
- **B–E. import + activate Curated V1 + deactivate alpha** — one
  transaction, as above.
- **F. validate final feed** — `verify_curated_v1_railway` (read-only, runs
  after commit) plus the API checks in §9.

## 8 — Post-cutover SQL checks

All of the following are exactly what `verify_curated_v1_railway` asserts in
one run (source reviewed in the prior pass); reproduced here as the prepared
manual-fallback SQL from `CURATED_V1_RAILWAY_IMPORT_PLAN.md` §"SQL
post-import acceptance checks":

- Curated works = 73, Curated pages = 12,025
- EN works = 26, FR works = 26, ES works = 21
- `content_v2 IS NULL` count = 0 for Curated V1 pages
- Page sequences continuous per edition, duplicate `(edition_id,
  page_number)` = 0
- Deterministic UUIDv5 work IDs match the manifest exactly (set equality)
- `content_hash`/`canonical_content_hash` match the manifest's per-page hash
  exactly
- Active non-Curated pages = 0 (i.e. alpha fully deactivated)
- User tables (`users`, `user_language_preferences`, `taste_profiles`,
  `user_affinities`, `user_pair_affinities`) row counts unchanged from the
  pre-cutover audit
- Zero FK violations (implied — no row is ever deleted, so no FK can break)

## 9 — Post-cutover API checks

Prepared, read-only, **not executed**:

```
curl.exe -i https://bookfin-api-production.up.railway.app/health
# then, with disposable anonymous identities only (no writes beyond that):
curl.exe -s https://bookfin-api-production.up.railway.app/api/v1/pages/random?language=fr
curl.exe -s https://bookfin-api-production.up.railway.app/api/v1/pages/random?language=en
curl.exe -s https://bookfin-api-production.up.railway.app/api/v1/pages/random?language=es
```

Each response must be checked for a populated `blocks` field (V2), a
`content_hash`, and the requested `language_tag`. Feed-exhaustion behavior
(the `feed_exhausted` JSON body added in this same commit, see
`src/web/api_v1.rs`) should be spot-checked by exhausting one language for a
disposable test identity — not the shared alpha identities. No production
reactions will be created solely for this test, per your instruction.

## 10 — Rollback trigger and exact restoration commands

Rollback immediately if: migration failure; import count ≠ 73 works or
≠ 12,025 pages; any hash mismatch; any FK failure; Curated feed
inaccessible; `/health` fails; or alpha/Curated feed eligibility cannot be
controlled safely (e.g. the postcondition assertions inside the importer's
own transaction fail — in which case Postgres rolls the transaction back
automatically and production is simply left unchanged, still on alpha).

If a rollback is needed **after** a successful commit (the only case that
needs the backup — a pre-commit failure self-rolls-back with no data
change), the prepared restoration is:

```powershell
# Requires the same SSH tunnel as the audit/backup pass:
railway connect Postgres --environment production --ssh --tunnel-only --port 55432
# In a second terminal, with the tunnel's printed URL as $TUNNEL_URL:
& "C:\Program Files\PostgreSQL\18\bin\pg_restore.exe" --clean --if-exists `
  --dbname="$TUNNEL_URL" `
  "$env:USERPROFILE\Bookfin-Railway-Backups\bookfin-pre-curated-v1-20260916-184148.dump"
```

This is destructive to whatever is live in Railway at the moment it runs (it
`--clean`s existing objects before restoring), so it must only be run as a
deliberate incident action after explicit authorization — never automatically
as part of this plan.

## 11 — Final execution gate

All commands for the cutover are prepared and documented above (§3, §5, §6,
§8, §9); none were executed against Railway beyond the read-only audit and
`pg_dump` from the previous pass, plus this pass's `git push` (code only, no
data or schema change). Per your instruction, the cutover itself — deploying
the reviewed commit and running `ingest_curated_v1 --apply-railway` — is
withheld until you send the explicit instruction **"EXECUTE THE RAILWAY
CUTOVER."** EAS was not launched.

READY FOR CURATED V1 CUTOVER: **YES**

No technical or safety blocker remains: backup is validated and reachable,
migrations are confirmed additive and low-risk at current scale, the
importer/verifier enforce every required invariant atomically, user-data FK
and history behavior is understood and safe, and the repository now has a
reviewed, pushed commit on a real remote. The only remaining gate is your
explicit production authorization.
