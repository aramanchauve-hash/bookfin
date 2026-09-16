#!/usr/bin/env python3
"""Regression checks for the evidence reported in CURATED_V1_SPANISH_SOURCE_AUDIT."""

from __future__ import annotations

import importlib.util
import io
import json
from contextlib import redirect_stdout
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
SCAN = ROOT / "scripts" / "scan_spanish_corpus_anomalies.py"


def scan() -> list[dict]:
    spec = importlib.util.spec_from_file_location("spanish_scan", SCAN)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    output = io.StringIO()
    with redirect_stdout(output):
        module.main()
    return json.loads(output.getvalue())


def main() -> None:
    works = {work["work_id"]: work for work in scan()}
    assert len(works) == 21, f"expected 21 Spanish works, found {len(works)}"

    quijote = works["es-cervantes-quijote"]
    assert quijote["source_has_toc"]
    assert quijote["normalized_glued_case"] == 0
    assert quijote["paged_glued_case"] == 0

    niebla = works["es-unamuno-niebla"]
    assert niebla["source_page_markers"] > 0
    # The remaining p. 268 is a transcriber's sentence, not a pagenum span.
    assert niebla["normalized_page_markers"] == 1
    assert niebla["paged_page_markers"] == 1

    luces = works["es-valle-inclan-luces-de-bohemia"]
    assert luces["source_page_markers"] > 0
    assert luces["normalized_page_markers"] == 0
    assert luces["paged_page_markers"] == 0
    print("Spanish Curated V1 source audit checks passed (21 works).")


if __name__ == "__main__":
    main()
