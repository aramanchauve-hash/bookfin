#!/usr/bin/env python3
"""Read-only quality scan of all Bookfin Curated V1 works.

The scanner never writes below ``corpus/``. It compares source HTML,
normalized V2 document and paginated V2 output, then writes a human review
report under ``docs/reports``. A lexical regex is never sufficient to mark a
work BLOCK: BLOCK is reserved for missing/corrupt canonical artifacts or a
broken pagination invariant.
"""

from __future__ import annotations

import json
import re
from collections import Counter
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any, Iterable


ROOT = Path(__file__).resolve().parents[1]
CURATED = ROOT / "corpus" / "curated_v1"
MANIFEST_PATH = CURATED / "manifest.json"
REPORT_PATH = ROOT / "docs" / "reports" / "CURATED_V1_GLOBAL_SANITY_SCAN.md"

MARKERS: tuple[tuple[str, re.Pattern[str]], ...] = (
    ("BRACKET_PAGE", re.compile(r"\[(?:pg|page|p[áa]g(?:ina)?)\.?\s*\d+\]", re.I)),
    ("PAGE_MARKER", re.compile(r"\b(?:pg|p[áa]g|p)\.\s*\d+\b", re.I)),
    ("FACSIMILE", re.compile(r"\bfac-?simil(?:e|é)\b", re.I)),
)
BOILERPLATE: tuple[tuple[str, re.Pattern[str]], ...] = (
    ("GUTENBERG", re.compile(r"\b(?:project gutenberg|gutenberg-tm|end of the project gutenberg|distributed proofreaders?)\b", re.I)),
    ("TOC", re.compile(r"^\s*(?:table of contents|table des mati[eè]res|[íi]ndice general|tabla de materias)\s*$", re.I | re.M)),
    ("HTML_NAV", re.compile(r"\b(?:next chapter|previous chapter|chapitre suivant|p[áa]gina siguiente)\b", re.I)),
)
STRUCTURAL: tuple[tuple[str, re.Pattern[str]], ...] = (
    ("HTML_TAG", re.compile(r"</?[a-z][a-z0-9]*\b[^>]*>", re.I)),
    ("HTML_ENTITY", re.compile(r"&(?:amp|lt|gt|quot|apos|nbsp|#\d+|#x[0-9a-f]+);", re.I)),
    ("CONTROL_CHAR", re.compile(r"[\x00-\x08\x0b\x0c\x0e-\x1f\x7f]")),
    ("SOFT_HYPHEN", re.compile("\u00ad")),
    ("REPLACEMENT_CHAR", re.compile("\ufffd")),
)
GLUED_CASE = re.compile(r"(?<=[a-zà-öø-ÿ])(?=[A-ZÀ-ÖØ-Þ])")
NAME_PREFIXES = {"mc", "mac", "o", "d", "l"}


@dataclass(frozen=True)
class Hit:
    signature: str
    layer: str
    page: int | None
    block: int | None
    context: str
    false_positive: bool = False
    rationale: str = ""


@dataclass
class Result:
    work_id: str
    language: str
    title: str
    pages: int = 0
    status: str = "PASS"
    hits: list[Hit] = field(default_factory=list)
    blockers: list[str] = field(default_factory=list)


def excerpt(text: str, start: int, end: int) -> str:
    return " ".join(text[max(0, start - 36): min(len(text), end + 60)].split())[:160]


def block_text(block: dict[str, Any]) -> str:
    return "".join(str(span.get("text", "")) for span in block.get("spans", []))


def document_blocks(document: dict[str, Any]) -> list[dict[str, Any]]:
    return document.get("document", document).get("blocks", [])


def page_blocks(pages: Iterable[dict[str, Any]]) -> Iterable[tuple[int, int, dict[str, Any]]]:
    for page_index, page in enumerate(pages, start=1):
        sequence = page.get("page_sequence_number", page_index)
        for block_index, block in enumerate(page.get("blocks", [])):
            yield int(sequence) if isinstance(sequence, int) else page_index, block_index, block


def lexical_hits(text: str, layer: str, page: int | None, block: int | None) -> list[Hit]:
    hits: list[Hit] = []
    for signature, pattern in (*MARKERS, *BOILERPLATE, *STRUCTURAL):
        match = pattern.search(text)
        if match:
            hits.append(Hit(signature, layer, page, block, excerpt(text, match.start(), match.end())))

    # Camel-case concatenation is only a REVIEW candidate; named particles are
    # excluded because they are normal orthography (MacDonald, d'Artagnan, ...).
    for token in re.findall(r"\b[A-Za-zÀ-ÖØ-öø-ÿ]{4,}\b", text):
        match = GLUED_CASE.search(token)
        if match and token[:match.start()].lower() not in NAME_PREFIXES:
            hits.append(Hit("GLUED_CASE", layer, page, block, token[:120]))
            break
    return hits


def read_json(path: Path, result: Result, label: str) -> dict[str, Any] | None:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError:
        result.blockers.append(f"MISSING_{label}:{path.relative_to(ROOT).as_posix()}")
    except (OSError, json.JSONDecodeError) as error:
        result.blockers.append(f"UNREADABLE_{label}:{path.relative_to(ROOT).as_posix()} ({error})")
    return None


def audit_work(meta: dict[str, Any]) -> Result:
    work_id = meta["work_id"]
    language = meta["language"]
    result = Result(work_id, language, meta.get("title", ""))
    source_path = CURATED / "sources" / language / f"{work_id}.html"
    normalized_path = CURATED / "normalized" / language / f"{work_id}.json"
    pages_path = CURATED / "pages" / language / f"{work_id}_pages.json"

    try:
        source_text = source_path.read_text(encoding="utf-8")
    except FileNotFoundError:
        result.blockers.append(f"MISSING_SOURCE:{source_path.relative_to(ROOT).as_posix()}")
        source_text = ""
    except OSError as error:
        result.blockers.append(f"UNREADABLE_SOURCE:{source_path.relative_to(ROOT).as_posix()} ({error})")
        source_text = ""
    source_hits = lexical_hits(source_text, "SOURCE", None, None)

    normalized = read_json(normalized_path, result, "NORMALIZED")
    pages_doc = read_json(pages_path, result, "PAGES")
    normalized_hits: list[Hit] = []
    page_hits: list[Hit] = []
    if normalized is not None:
        for index, block in enumerate(document_blocks(normalized)):
            normalized_hits.extend(lexical_hits(block_text(block), "NORMALIZER", None, index))
    if pages_doc is not None:
        pages = pages_doc.get("pages", [])
        result.pages = len(pages)
        declared = pages_doc.get("pages_count")
        if declared != len(pages):
            result.blockers.append(f"PAGES_COUNT_MISMATCH: declared={declared}, actual={len(pages)}")
        hashes: set[str] = set()
        for index, page in enumerate(pages, start=1):
            if page.get("page_sequence_number") != index:
                result.blockers.append(f"DISCONTINUOUS_SEQUENCE: index={index}, sequence={page.get('page_sequence_number')}")
            if not page.get("blocks"):
                result.blockers.append(f"EMPTY_PAGE: page={index}")
            content_hash = page.get("content_hash")
            if not isinstance(content_hash, str) or not content_hash:
                result.blockers.append(f"MISSING_CONTENT_HASH: page={index}")
            elif content_hash in hashes:
                result.blockers.append(f"DUPLICATE_CONTENT_HASH: page={index}")
            else:
                hashes.add(content_hash)
        for page, index, block in page_blocks(pages):
            page_hits.extend(lexical_hits(block_text(block), "PAGINATOR", page, index))

    # Source matches describe the input only. They are never enough to label
    # the published V2 corpus REVIEW/BLOCK: their possible survival is audited
    # independently in normalized_hits and page_hits below.
    result.hits.extend(
        Hit(hit.signature, hit.layer, hit.page, hit.block, hit.context, True, "source-only candidate; canonical V2 scanned separately")
        for hit in source_hits
    )
    result.hits.extend(normalized_hits)
    result.hits.extend(page_hits)
    if result.blockers:
        result.status = "BLOCK"
    elif any(not hit.false_positive for hit in result.hits):
        result.status = "REVIEW"
    return result


def format_hits(result: Result) -> str:
    real = Counter(hit.signature for hit in result.hits if not hit.false_positive)
    if result.blockers:
        return "; ".join(result.blockers)
    return ", ".join(f"{name}×{count}" for name, count in sorted(real.items())) or "—"


def probable_layer(result: Result) -> str:
    if result.blockers:
        return "CANONICAL"
    real_layers = {hit.layer for hit in result.hits if not hit.false_positive}
    if not real_layers:
        return "NONE"
    if real_layers == {"NORMALIZER"}:
        return "NORMALIZER"
    if real_layers == {"PAGINATOR"}:
        return "PAGINATOR"
    return "SOURCE/NORMALIZER"


def write_report(results: list[Result]) -> None:
    total_pages = sum(result.pages for result in results)
    counts = Counter(result.status for result in results)
    signature_counts = Counter(
        hit.signature for result in results for hit in result.hits if not hit.false_positive
    )
    fp_count = sum(1 for result in results for hit in result.hits if hit.false_positive)
    lines = [
        "# Curated V1 — Global Sanity Scan",
        "",
        "Read-only scan of source HTML → normalized V2 JSON → paginated V2 JSON. "
        "No file under `corpus/` is written by this command.",
        "",
        "## Result",
        "",
        f"- Works: **{len(results)}**; paginated pages: **{total_pages}**",
        f"- PASS: **{counts['PASS']}**; REVIEW: **{counts['REVIEW']}**; BLOCK: **{counts['BLOCK']}**",
        "- BLOCK is reserved for a missing/unreadable canonical artifact or a failed pagination invariant; a regex candidate alone cannot block release.",
        f"- Source-only false positives filtered by V2: **{fp_count}**",
        "",
        "## Top REVIEW candidates",
        "",
    ]
    ranked_reviews = sorted(
        (result for result in results if result.status == "REVIEW"),
        key=lambda result: (-sum(not hit.false_positive for hit in result.hits), result.work_id),
    )[:5]
    if ranked_reviews:
        lines.extend(
            f"- `{result.work_id}`: {format_hits(result)}"
            for result in ranked_reviews
        )
    else:
        lines.append("None.")
    lines.extend([
        "",
        "## Per-work classification",
        "",
        "| Work | Lang | Status | Anomalies / invariant | Probable layer | Signature count | Source-only FP |",
        "| --- | --- | --- | --- | --- | ---: | ---: |",
    ])
    for result in results:
        real = [hit for hit in result.hits if not hit.false_positive]
        fps = [hit for hit in result.hits if hit.false_positive]
        lines.append(
            f"| `{result.work_id}` | {result.language} | **{result.status}** | {format_hits(result)} | "
            f"{probable_layer(result)} | {len(real)} | {len(fps)} |"
        )
    lines.extend(["", "## REVIEW evidence", ""])
    reviews = [result for result in results if result.status == "REVIEW"]
    if not reviews:
        lines.append("None.")
    for result in reviews:
        evidence = next(hit for hit in result.hits if not hit.false_positive)
        page = f", page {evidence.page}" if evidence.page is not None else ""
        lines.append(f"- `{result.work_id}` — {evidence.signature}, {evidence.layer}{page}: “{evidence.context}”")
    lines.extend(["", "## BLOCK evidence", ""])
    blockers = [result for result in results if result.status == "BLOCK"]
    if not blockers:
        lines.append("None.")
    for result in blockers:
        lines.append(f"- `{result.work_id}`: " + "; ".join(result.blockers))
    lines.extend(["", "## Signature totals (canonical V2 only)", ""])
    if signature_counts:
        lines.extend(f"- `{signature}`: {count}" for signature, count in sorted(signature_counts.items()))
    else:
        lines.append("None.")
    lines.append("")
    REPORT_PATH.parent.mkdir(parents=True, exist_ok=True)
    REPORT_PATH.write_text("\n".join(lines), encoding="utf-8")


def main() -> None:
    manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))
    works = manifest["works"]
    if len(works) != 73:
        raise SystemExit(f"Expected 73 manifest works, found {len(works)}")
    results = [audit_work(meta) for meta in works]
    write_report(results)
    counts = Counter(result.status for result in results)
    print(f"Scanned {len(results)} works / {sum(result.pages for result in results)} pages")
    print(f"PASS={counts['PASS']} REVIEW={counts['REVIEW']} BLOCK={counts['BLOCK']}")
    print(REPORT_PATH.relative_to(ROOT))


if __name__ == "__main__":
    main()
