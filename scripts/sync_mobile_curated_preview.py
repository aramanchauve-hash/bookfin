#!/usr/bin/env python3
"""Create the small, reproducible Curated V1 fixture set used by Expo DEV.

The corpus directory remains the source of truth. This script copies the
mobile fixture verbatim and extracts one already-paginated V2 page per
additional typography case; it never rewrites literary blocks or spans.
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[1]
CORPUS_ROOT = ROOT / "corpus" / "curated_v1"
DESTINATION = ROOT / "mobile" / "src" / "fixtures" / "curated_v1"

PAGE_FIXTURES = (
    ("pages/en/en-james-turn-of-the-screw_pages.json", "turn_of_the_screw_pages.json", "PROLOGUE"),
    ("pages/en/en-hazlitt-table-talk_pages.json", "hazlitt_table_talk_pages.json", "Pure in the last recesses"),
    ("pages/fr/fr-huysmans-a-rebours_pages.json", "a_rebours_pages.json", "À REBOURS"),
    ("pages/fr/fr-barbey-les-diaboliques_pages.json", "les_diaboliques_pages.json", "AVERTISSEMENT"),
    ("pages/es/es-unamuno-niebla_pages.json", "niebla_pages.json", "VÍCTOR GOTI"),
    ("pages/es/es-valle-inclan-luces-de-bohemia_pages.json", "luces_de_bohemia_pages.json", "ESCENA PRIMERA"),
    ("pages/es/es-cervantes-quijote_pages.json", "don_quijote_pages.json", "TASA"),
    ("pages/es/es-larra-el-doncel_pages.json", "el_doncel_pages.json", "CAPITULO I"),
)


def read_json(path: Path) -> Any:
    with path.open("r", encoding="utf-8") as handle:
        return json.load(handle)


def page_contains(page: dict[str, Any], fragment: str) -> bool:
    needle = fragment.upper()
    return any(
        needle in span.get("text", "").upper()
        for block in page.get("blocks", [])
        for span in block.get("spans", [])
    )


def write_json(path: Path, data: Any) -> None:
    path.write_text(json.dumps(data, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def extract_page(source: Path, fragment: str) -> dict[str, Any]:
    work = read_json(source)
    page = next((item for item in work["pages"] if page_contains(item, fragment)), None)
    if page is None:
        raise ValueError(f"No V2 page containing {fragment!r} in {source}")

    # Preserve the selected page byte-for-byte in data terms: all original V2
    # fields, blocks and spans remain intact. Only the enclosing pages array is
    # reduced so the Expo bundle stays small.
    return {
        key: work[key]
        for key in (
            "work_id",
            "author",
            "title",
            "language",
            "work_kind",
            "edition_id",
            "source_sha256",
            "document_sha256",
        )
        if key in work
    } | {
        "pages_count": 1,
        "source_page_sequence_number": page["page_sequence_number"],
        "pages": [page],
    }


def refresh_base_samples(base: dict[str, Any], selected_work_ids: set[str] | None) -> dict[str, Any]:
    """Refresh only selected sample blocks from their regenerated V2 pages."""
    for sample in base.get("samples", []):
        work_id = sample.get("work_id")
        if selected_work_ids is not None and work_id not in selected_work_ids:
            continue
        source = CORPUS_ROOT / "pages" / sample["language"] / f"{work_id}_pages.json"
        if not source.exists():
            continue
        pages = read_json(source)["pages"]
        page = next((page for page in pages if page["page_sequence_number"] == sample["page_sequence_number"]), None)
        if page is None:
            raise ValueError(f"No page {sample['page_sequence_number']} for base sample {work_id}")
        sample["blocks"] = page["blocks"]
    return base


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--work-id", action="append", dest="work_ids")
    args = parser.parse_args()
    selected_work_ids = set(args.work_ids) if args.work_ids else None
    DESTINATION.mkdir(parents=True, exist_ok=True)
    base = refresh_base_samples(read_json(CORPUS_ROOT / "mobile_preview_fixture.json"), selected_work_ids)
    if selected_work_ids is None or any(sample.get("work_id") in selected_work_ids for sample in base.get("samples", [])):
        write_json(DESTINATION / "mobile_preview_fixture.json", base)

    for source_relative, destination_name, fragment in PAGE_FIXTURES:
        work_id = Path(source_relative).name.removesuffix("_pages.json")
        if selected_work_ids is not None and work_id not in selected_work_ids:
            continue
        fixture = extract_page(CORPUS_ROOT / source_relative, fragment)
        write_json(DESTINATION / destination_name, fixture)
        print(f"synced {destination_name}: page {fixture['source_page_sequence_number']}")


if __name__ == "__main__":
    main()
