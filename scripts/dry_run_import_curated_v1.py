#!/usr/bin/env python3
"""Prepare, but never execute, a Curated V1 database import.

This is deliberately a filesystem-only dry run: it reads canonical V2 JSON
and writes a deterministic import plan below ``target/``.  It does not accept
database credentials, import sqlx, open a network connection, or mutate
``corpus/``.  The plan is an input to the future transactional importer.
"""

from __future__ import annotations

import hashlib
import json
import uuid
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[1]
CURATED = ROOT / "corpus" / "curated_v1"
OUT = ROOT / "target" / "curated_v1_import_dry_run.json"
NAMESPACE = uuid.UUID("e3b7724e-72fe-54cf-a53f-cbc1d6f4f6d0")


def canonical_json(value: Any) -> str:
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":"))


def flatten_blocks(blocks: list[dict[str, Any]]) -> str:
    return "\n\n".join(
        "".join(str(span.get("text", "")) for span in block.get("spans", [])).strip()
        for block in blocks
        if "".join(str(span.get("text", "")) for span in block.get("spans", [])).strip()
    )


def uuid_for(kind: str, *parts: str) -> str:
    return str(uuid.uuid5(NAMESPACE, ":".join((kind, *parts))))


def sha256(text: str) -> str:
    return hashlib.sha256(text.encode("utf-8")).hexdigest()


def main() -> None:
    manifest_bytes = (CURATED / "manifest.json").read_bytes()
    manifest = json.loads(manifest_bytes)
    works = manifest["works"]
    if len(works) != 73:
        raise SystemExit(f"expected exactly 73 works, got {len(works)}")

    plan_works: list[dict[str, Any]] = []
    total_pages = 0
    for meta in works:
        work_id = meta["work_id"]
        language = meta["language"]
        source_path = CURATED / "sources" / language / f"{work_id}.html"
        normalized_path = CURATED / "normalized" / language / f"{work_id}.json"
        pages_path = CURATED / "pages" / language / f"{work_id}_pages.json"
        if not source_path.is_file() or not normalized_path.is_file() or not pages_path.is_file():
            raise SystemExit(f"missing canonical artifact for {work_id}")
        normalized = json.loads(normalized_path.read_text(encoding="utf-8"))
        paginated = json.loads(pages_path.read_text(encoding="utf-8"))
        pages = paginated.get("pages", [])
        if len(pages) != paginated.get("pages_count") or not pages:
            raise SystemExit(f"invalid page count for {work_id}")
        if normalized.get("document_sha256") != paginated.get("document_sha256"):
            raise SystemExit(f"document hash mismatch for {work_id}")

        planned_pages = []
        for sequence, page in enumerate(pages, start=1):
            if page.get("page_sequence_number") != sequence:
                raise SystemExit(f"non-contiguous page sequence for {work_id}")
            blocks = page.get("blocks")
            if not isinstance(blocks, list) or not blocks:
                raise SystemExit(f"empty V2 blocks for {work_id} page {sequence}")
            canonical_hash = page.get("content_hash")
            if not isinstance(canonical_hash, str) or len(canonical_hash) != 64:
                raise SystemExit(f"missing V2 content hash for {work_id} page {sequence}")
            fallback_text = flatten_blocks(blocks)
            if not fallback_text:
                raise SystemExit(f"empty fallback text for {work_id} page {sequence}")
            planned_pages.append({
                "id": uuid_for("page", work_id, str(sequence), "v2"),
                "page_number": sequence,
                "version": 2,
                "canonical_content_hash": canonical_hash,
                "legacy_text_hash": sha256(" ".join(fallback_text.split())),
                "block_json_sha256": sha256(canonical_json(blocks)),
                "token_count": len(fallback_text.split()),
                "block_count": len(blocks),
            })
        total_pages += len(planned_pages)
        plan_works.append({
            "work_id": work_id,
            "work_row_id": uuid_for("work", work_id),
            "edition_row_id": uuid_for("edition", work_id, "curated-v1"),
            "language": language,
            "title": meta["title"],
            "author": meta["author"],
            "work_kind": meta["work_kind"],
            "source_sha256": normalized["source_sha256"],
            "document_sha256": normalized["document_sha256"],
            "pages": planned_pages,
        })

    if total_pages != manifest["total_real_pages"]:
        raise SystemExit(f"expected {manifest['total_real_pages']} pages, got {total_pages}")
    plan = {
        "kind": "bookfin.curated-v1.import-dry-run.v1",
        "database_access": "none",
        "manifest_sha256": hashlib.sha256(manifest_bytes).hexdigest(),
        "manifest_version": manifest["manifest_version"],
        "works": len(plan_works),
        "pages": total_pages,
        "required_schema_migration": "0011_page_content_v2.sql",
        "idempotency": "UUIDv5(namespace, kind/work_id/page_sequence/version)",
        "work_plan": plan_works,
    }
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(canonical_json(plan) + "\n", encoding="utf-8")
    print(f"DRY RUN ONLY: {len(plan_works)} works, {total_pages} pages")
    print(f"No database connection opened. Plan: {OUT.relative_to(ROOT)}")


if __name__ == "__main__":
    main()
