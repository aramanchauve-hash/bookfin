# Curated V1 — Future import and rollback runbook

This is a runbook for a later, explicitly authorized environment. It is not an
instruction to run production commands now.

## Preconditions

1. Confirm the manifest SHA-256 is
   `bd9c3e749efd4d499b5c6db876fb565ece4a7c09f0ce21b59a46377ec3b71528`.
2. Confirm the schema/API work listed in
   `docs/reports/CURATED_V1_SCHEMA_IMPORT_DRY_RUN.md` is merged and tested.
3. Take a database backup/snapshot and record the target database identity.
4. Run the local dry run and obtain exactly 73 works and 12,025 pages.
5. Start with an isolated staging database. Do not set a production URL until
   staging validation has been reviewed.

## Future staged procedure

1. Apply only forward migrations using the normal migration runner.
2. Run the importer in a single transaction, scoped to the deterministic
   Curated V1 edition IDs.
3. Before commit, assert 73 works, 73 editions and 12,025 active pages; assert
   each page's V2 canonical hash and block JSON hash against the dry-run plan.
4. Query representative EN/FR/ES pages through the API and verify that `blocks`
   is present and `text` remains a fallback.
5. Commit only after those assertions pass; capture the import log and the
   manifest hash with the release record.

## Rollback

If the transaction has not committed, roll it back. After a committed import,
do not delete or overwrite served pages: use the recorded Curated V1 edition
IDs to mark the imported edition/pages inactive, then restore the pre-import
backup only if that is the approved incident action. Preserve impressions and
reactions; the immutability rule in migration 0006 exists specifically to
prevent rewriting their historical content.

## Never do

- Do not delete rows from `_sqlx_migrations`.
- Do not repaginate during import.
- Do not substitute flattened text for V2 blocks.
- Do not run this procedure against Railway without a separate authorization.
