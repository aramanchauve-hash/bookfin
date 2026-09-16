#!/usr/bin/env python3
"""Read-only structural audit for the 21 Spanish Curated V1 works.

The scan deliberately reads the stored source, normalized document and
paginated pages.  It never rebuilds or rewrites the corpus.  Its compact JSON
output is suitable for reviewing source/normalizer/paginator propagation.
"""

from __future__ import annotations

import json
import re
from collections import Counter
from pathlib import Path
from typing import Any, Iterable


ROOT = Path(__file__).resolve().parents[1]
CURATED = ROOT / "corpus" / "curated_v1"
AUDIT = ROOT / "corpus" / "curation" / "audit_73_candidates.json"
INTEGRITY = CURATED / "integrity_audit_report.json"

PAGE_MARKER = re.compile(r"\[(?:Pg|Page)\s*\d+\]|\bp\.\s*\d+", re.IGNORECASE)
GLUED_CASE = re.compile(r"(?<=[a-záéíóúüñ])(?=[A-ZÁÉÍÓÚÜÑ])")


def text_of(block: dict[str, Any]) -> str:
    return "".join(span.get("text", "") for span in block.get("spans", []))


def all_page_blocks(pages: Iterable[dict[str, Any]]) -> Iterable[dict[str, Any]]:
    for page in pages:
        yield from page.get("blocks", [])


def examples(text: str, expression: re.Pattern[str], limit: int = 3) -> list[str]:
    result: list[str] = []
    for match in expression.finditer(text):
        start, end = max(0, match.start() - 48), min(len(text), match.end() + 64)
        result.append(re.sub(r"\s+", " ", text[start:end]).strip())
        if len(result) == limit:
            break
    return result


def source_format(url: str) -> str:
    return "Wikisource HTML" if "wikisource.org" in url else "Project Gutenberg HTML"


def audit_work(candidate: dict[str, Any], integrity: dict[str, Any]) -> dict[str, Any]:
    work_id = candidate["id"]
    source = (CURATED / "sources" / "es" / f"{work_id}.html").read_text(encoding="utf-8")
    normalized = json.loads((CURATED / "normalized" / "es" / f"{work_id}.json").read_text(encoding="utf-8"))
    pages = json.loads((CURATED / "pages" / "es" / f"{work_id}_pages.json").read_text(encoding="utf-8"))["pages"]
    document_blocks = normalized["document"]["blocks"]
    normalized_text = "\n".join(text_of(block) for block in document_blocks)
    paged_text = "\n".join(text_of(block) for block in all_page_blocks(pages))

    def marker_count(value: str) -> int:
        return len(PAGE_MARKER.findall(value))

    def glued_count(value: str) -> int:
        return len(GLUED_CASE.findall(value))

    block_types = Counter(block.get("type", "unknown") for block in document_blocks)
    samples = [0, len(pages) // 2, len(pages) - 1]
    page_samples = []
    for index in sorted(set(samples)):
        sample_text = " ".join(text_of(block) for block in pages[index].get("blocks", []))
        page_samples.append({"page": index + 1, "excerpt": re.sub(r"\s+", " ", sample_text)[:180]})

    return {
        "work_id": work_id,
        "title": candidate["title"],
        # The acquisition record, not the pre-curation candidate, is the
        # authoritative description of the bytes in curated_v1/sources.
        "source_url": integrity["source"]["download_url"],
        "source_format": source_format(integrity["source"]["download_url"]),
        "source_page_markers": marker_count(source),
        "normalized_page_markers": marker_count(normalized_text),
        "paged_page_markers": marker_count(paged_text),
        "normalized_glued_case": glued_count(normalized_text),
        "paged_glued_case": glued_count(paged_text),
        "source_has_toc": bool(re.search(r"(?:id|class)=[\"'][^\"']*toc", source, re.IGNORECASE)),
        "normalized_blocks": dict(sorted(block_types.items())),
        "marker_examples": examples(normalized_text, PAGE_MARKER),
        "glued_examples": examples(normalized_text, GLUED_CASE),
        "page_samples": page_samples,
    }


def main() -> None:
    candidates = json.loads(AUDIT.read_text(encoding="utf-8"))
    spanish = [candidate for candidate in candidates if candidate["id"].startswith("es-")]
    integrity = {
        item["work_id"]: item
        for item in json.loads(INTEGRITY.read_text(encoding="utf-8"))
        if item["work_id"].startswith("es-")
    }
    result = [audit_work(candidate, integrity[candidate["id"]]) for candidate in spanish]
    print(json.dumps(result, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
