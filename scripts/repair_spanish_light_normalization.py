#!/usr/bin/env python3
"""Surgically rebuild the ten Spanish light-normalization targets.

The archived HTML source is intentionally read-only.  A guard snapshot must
be captured before regeneration; it makes an accidental change outside the
ten approved Spanish targets a hard failure.
"""

from __future__ import annotations

import argparse
import hashlib
import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

from build_curated_v1 import (
    SPECIAL_URL_OVERRIDES,
    canonical_json,
    generate_control_samples,
    normalize_html_to_document,
    paginate_document,
    sha256_text,
)


ROOT = Path(__file__).resolve().parents[1]
CURATED = ROOT / "corpus" / "curated_v1"
CANDIDATES = ROOT / "corpus" / "curation" / "library_v1_candidates.json"
TARGETS = (
    "es-cervantes-quijote",
    "es-cervantes-rinconete-y-cortadillo",
    "es-cervantes-la-gitanilla",
    "es-cervantes-el-licenciado-vidriera",
    "es-cervantes-el-celoso-extremeno",
    "es-cervantes-el-coloquio-de-los-perros",
    "es-quevedo-los-suenos",
    "es-larra-el-doncel",
    "es-unamuno-niebla",
    "es-valle-inclan-luces-de-bohemia",
)
GUARD_PATH = CURATED / "spanish_light_normalization_guard.json"


def sha256_file(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def load_candidates() -> dict[str, dict[str, Any]]:
    records = json.loads(CANDIDATES.read_text(encoding="utf-8"))["candidates"]
    return {record["id"]: record for record in records}


def paths_for(work_id: str, language: str) -> dict[str, Path]:
    return {
        "source": CURATED / "sources" / language / f"{work_id}.html",
        "normalized": CURATED / "normalized" / language / f"{work_id}.json",
        "pages": CURATED / "pages" / language / f"{work_id}_pages.json",
        "samples": CURATED / "samples" / language / f"{work_id}_samples.json",
    }


def capture_guard() -> None:
    candidates = load_candidates()
    accepted = [candidate for candidate in candidates.values() if candidate["status"] == "ACCEPTED"]
    if len(accepted) != 73:
        raise RuntimeError(f"expected 73 ACCEPTED works, found {len(accepted)}")
    snapshot: dict[str, dict[str, str]] = {}
    for candidate in accepted:
        work_id, language = candidate["id"], candidate["original_language"]
        snapshot[work_id] = {
            name: sha256_file(path)
            for name, path in paths_for(work_id, language).items()
            if path.exists()
        }
    payload = {
        "format": "bookfin.spanish-light-normalization-guard.v1",
        "targets": list(TARGETS),
        "accepted_work_ids": sorted(snapshot),
        "files": snapshot,
    }
    GUARD_PATH.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(f"Captured guard for {len(snapshot)} works: {GUARD_PATH.relative_to(ROOT)}")


def assert_guard(*, after_regeneration: bool) -> None:
    if not GUARD_PATH.exists():
        raise RuntimeError("guard is missing; run with --capture-guard before regeneration")
    guard = json.loads(GUARD_PATH.read_text(encoding="utf-8"))
    if tuple(guard["targets"]) != TARGETS:
        raise RuntimeError("guard targets do not match the approved target list")
    candidates = load_candidates()
    accepted_ids = sorted(candidate["id"] for candidate in candidates.values() if candidate["status"] == "ACCEPTED")
    if accepted_ids != guard["accepted_work_ids"]:
        raise RuntimeError("ACCEPTED work IDs changed since the guard was captured")
    for work_id, file_hashes in guard["files"].items():
        candidate = candidates[work_id]
        current = paths_for(work_id, candidate["original_language"])
        protected = {"source"} if work_id in TARGETS else set(file_hashes)
        for name in protected:
            path = current[name]
            if sha256_file(path) != file_hashes[name]:
                stage = "after" if after_regeneration else "before"
                raise RuntimeError(f"{stage} regeneration guard failed: {work_id} {name} changed")
    print("Guard passed: all sources and all 63 non-target works are byte-identical.")


def make_manifest(candidates: dict[str, dict[str, Any]]) -> dict[str, Any]:
    accepted = [candidate for candidate in candidates.values() if candidate["status"] == "ACCEPTED"]
    works = []
    per_language = {"en": 0, "fr": 0, "es": 0}
    per_author: dict[str, int] = {}
    long_pages = short_pages = 0
    for candidate in accepted:
        work_id, language = candidate["id"], candidate["original_language"]
        pages_count = json.loads(paths_for(work_id, language)["pages"].read_text(encoding="utf-8"))["pages_count"]
        page_range = candidate["estimated_bookfin_pages"]
        first, last = (float(value.strip()) for value in page_range.split("-", 1))
        midpoint = (first + last) / 2
        per_language[language] += pages_count
        per_author[candidate["author"]] = per_author.get(candidate["author"], 0) + pages_count
        if candidate["work_kind"] == "LONG_FORM":
            long_pages += pages_count
        else:
            short_pages += pages_count
        works.append({
            "work_id": work_id,
            "author": candidate["author"],
            "title": candidate["title"],
            "language": language,
            "work_kind": candidate["work_kind"],
            "estimated_range": page_range,
            "estimated_pages_midpoint": midpoint,
            "real_pages_v2": pages_count,
            "diff_absolute": round(pages_count - midpoint, 1),
            "diff_percent": round((pages_count - midpoint) / midpoint * 100, 1),
        })
    total_pages = sum(per_language.values())
    for work in works:
        work["urn_share_percent"] = round(work["real_pages_v2"] / total_pages * 100, 3)
    authors = [
        {"author": author, "pages": pages, "percent": round(pages / total_pages * 100, 2)}
        for author, pages in sorted(per_author.items(), key=lambda item: -item[1])
    ]
    ranked = sorted(works, key=lambda work: -work["real_pages_v2"])
    return {
        "manifest_name": "Bookfin Curated Library V1",
        "manifest_version": "1.2-spanish-light-normalization",
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "total_works": len(works),
        "total_real_pages": total_pages,
        "linguistic_distribution": {
            language: {"pages": pages, "percent": round(pages / total_pages * 100, 2)}
            for language, pages in per_language.items()
        },
        "work_kind_distribution": {
            "LONG_FORM": {"count": 25, "pages": long_pages, "percent": round(long_pages / total_pages * 100, 2)},
            "SHORT_WORK": {"count": 48, "pages": short_pages, "percent": round(short_pages / total_pages * 100, 2)},
        },
        "top_15_authors": authors[:15],
        "top_15_works": [
            {key: work[key] for key in ("work_id", "author", "title", "real_pages_v2", "urn_share_percent")}
            for work in ranked[:15]
        ],
        "works": works,
    }


def source_html_for_target(work_id: str, source: str) -> str:
    """Apply the already-approved anthology boundaries before normalization.

    The five Cervantes novellas intentionally share one archived Gutenberg
    anthology.  Their existing V1 contract is the explicit start/end boundary
    stored in SPECIAL_URL_OVERRIDES; treating the whole anthology as each work
    would be a destructive over-expansion.
    """
    spec = SPECIAL_URL_OVERRIDES.get(work_id, {})
    if not (spec.get("slice_start") and spec.get("slice_end")):
        return source
    start_marker, end_marker = spec["slice_start"], spec["slice_end"]
    start = source.find(start_marker, spec.get("slice_start_offset", 0))
    if start == -1:
        raise RuntimeError(f"slice start not found for {work_id}: {start_marker!r}")
    heading_start = source.rfind("<h", max(0, start - 100), start)
    if heading_start != -1:
        start = heading_start
    end = source.find(end_marker, start + len(start_marker))
    if end == -1:
        raise RuntimeError(f"slice end not found for {work_id}: {end_marker!r}")
    heading_end = source.rfind("<h", max(0, end - 100), end)
    return source[start:heading_end if heading_end != -1 else end]


def repair() -> None:
    assert_guard(after_regeneration=False)
    candidates = load_candidates()
    changed: list[str] = []
    for work_id in TARGETS:
        candidate = candidates[work_id]
        language = candidate["original_language"]
        files = paths_for(work_id, language)
        old = json.loads(files["normalized"].read_text(encoding="utf-8"))
        source = files["source"].read_text(encoding="utf-8")
        document = normalize_html_to_document(
            source_html_for_target(work_id, source),
            source_format="gutenberg_html",
        )
        document_sha256 = sha256_text(canonical_json(document))
        normalized = {
            "work_id": work_id,
            "author": candidate["author"],
            "title": candidate["title"],
            "language": language,
            "work_kind": candidate["work_kind"],
            "edition_id": old["edition_id"],
            "document_sha256": document_sha256,
            "source_sha256": sha256_file(files["source"]),
            "document": document,
        }
        pages = paginate_document(
            document,
            work_id=work_id,
            edition_id=old["edition_id"],
            language=language,
            target_characters=1600,
        )
        page_payload = {
            "work_id": work_id,
            "author": candidate["author"],
            "title": candidate["title"],
            "language": language,
            "work_kind": candidate["work_kind"],
            "edition_id": old["edition_id"],
            "source_sha256": normalized["source_sha256"],
            "document_sha256": document_sha256,
            "pages_count": len(pages),
            "pages": pages,
        }
        files["normalized"].write_text(json.dumps(normalized, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        files["pages"].write_text(json.dumps(page_payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        files["samples"].write_text(
            json.dumps(generate_control_samples(candidate, pages), ensure_ascii=False, indent=2) + "\n",
            encoding="utf-8",
        )
        changed.append(f"{work_id}: {old['document_sha256'][:12]} -> {document_sha256[:12]}, {len(pages)} pages")

    manifest = make_manifest(candidates)
    manifest_path = CURATED / "manifest.json"
    manifest_path.write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    lock_path = CURATED / "curated_v1.lock.json"
    lock = json.loads(lock_path.read_text(encoding="utf-8"))
    if set(lock["accepted_work_ids"]) != {work["work_id"] for work in manifest["works"]}:
        raise RuntimeError("accepted_work_ids would change")
    lock["normalizer"]["version"] = "2.1.0"
    lock["updated_at"] = datetime.now(timezone.utc).isoformat()
    lock["total_real_pages"] = manifest["total_real_pages"]
    lock["manifest_sha256"] = sha256_file(manifest_path)
    lock_path.write_text(json.dumps(lock, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    assert_guard(after_regeneration=True)
    print("Repaired only approved targets:")
    print("\n".join(changed))


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--capture-guard", action="store_true")
    parser.add_argument("--repair", action="store_true")
    args = parser.parse_args()
    if args.capture_guard == args.repair:
        parser.error("choose exactly one of --capture-guard or --repair")
    if args.capture_guard:
        capture_guard()
    else:
        repair()


if __name__ == "__main__":
    main()
