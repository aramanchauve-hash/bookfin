#!/usr/bin/env python3
"""Targeted regression tests for the approved Spanish light normalization."""

from __future__ import annotations

import json
import re
from pathlib import Path

from build_curated_v1 import normalize_html_to_document
from repair_spanish_light_normalization import TARGETS, assert_guard


ROOT = Path(__file__).resolve().parents[1]
CURATED = ROOT / "corpus" / "curated_v1"


def normalized_text(work_id: str) -> str:
    payload = json.loads((CURATED / "normalized" / "es" / f"{work_id}.json").read_text(encoding="utf-8"))
    return "\n".join(
        "".join(span.get("text", "") for span in block.get("spans", []))
        for block in payload["document"]["blocks"]
    )


def paged_text(work_id: str) -> str:
    payload = json.loads((CURATED / "pages" / "es" / f"{work_id}_pages.json").read_text(encoding="utf-8"))
    return "\n".join(
        "".join(span.get("text", "") for span in block.get("spans", []))
        for page in payload["pages"]
        for block in page["blocks"]
    )


def main() -> None:
    # Source DOM → canonical block rules are explicit, not literary heuristics.
    pagenum = normalize_html_to_document('<p><span class="pagenum">p. 14</span>MAX</p>', source_format="gutenberg_html")
    assert pagenum["blocks"] == [{"type": "paragraph", "spans": [{"text": "MAX"}]}]
    toc = normalize_html_to_document(
        '<div class="toc">Title</div><ul><li>Entry</li></ul><p>Real prose.</p>',
        source_format="gutenberg_html",
    )
    assert "Real prose." == "".join(toc["blocks"][0]["spans"][0]["text"])

    assert_guard(after_regeneration=True)
    for work_id in TARGETS:
        text = normalized_text(work_id)
        pages = paged_text(work_id)
        assert "[Pg " not in text and "[Pg " not in pages, work_id
        assert "p. 14MAX" not in text and "p. 14MAX" not in pages, work_id
    quijote = normalized_text("es-cervantes-quijote")
    for artifact in ("letrasDonde", "aventurasQue", "sucesosDonde", "TasaTestimonio"):
        assert artifact not in quijote, artifact
    larra = normalized_text("es-larra-el-doncel")
    assert "The Project Gutenberg eBook" not in larra
    assert not re.search(r"\[p\.\s*\d+\]", larra, re.IGNORECASE)
    print("Spanish light-normalization regression checks passed (10 targets).")


if __name__ == "__main__":
    main()
