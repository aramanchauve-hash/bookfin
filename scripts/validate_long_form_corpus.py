#!/usr/bin/env python3
"""Offline integrity audit for Bookfin Long-Form Corpus 01."""

from __future__ import annotations

import hashlib
import json
import re
import sys
import unicodedata
from collections import Counter
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
MANIFEST = ROOT / "corpus" / "long_form_01" / "manifest.json"


def folded(value: str) -> str:
    value = unicodedata.normalize("NFKD", value).encode("ascii", "ignore").decode("ascii")
    return re.sub(r"[^a-z0-9]+", " ", value.casefold()).strip()


def page_count(text: str, target: int = 1600) -> int:
    """Mirror the Rust pilot paginator's character/whitespace rule."""
    text = text.strip()
    if not text:
        return 0
    chars = list(text)
    cursor, pages, total = 0, 0, len(chars)
    while cursor < total:
        if total - cursor <= target * 5 // 4:
            return pages + 1
        target_index = cursor + target
        if target_index >= total:
            return pages + 1
        cut = next((i for i in range(target_index, min(target_index + 200, total)) if chars[i].isspace()), None)
        if cut is None:
            for i in range(target_index - 1, max(cursor + 99, target_index - 200) - 1, -1):
                if chars[i].isspace():
                    cut = i
                    break
        cursor = cut if cut is not None else target_index
        while cursor < total and chars[cursor].isspace():
            cursor += 1
        pages += 1
    return pages


def main() -> int:
    manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
    problems, seen_hashes, total_pages = [], set(), 0
    author_pages: Counter[str] = Counter()
    for work in manifest["works"]:
        path = ROOT / work["file_path"]
        if not path.is_file():
            problems.append(f"{work['slug']}: missing text file")
            continue
        text = path.read_text(encoding="utf-8")
        if not work.get("source_identity_checked"):
            problems.append(f"{work['slug']}: source identity was not checked before cleaning")
        digest = hashlib.sha256(text.encode("utf-8")).hexdigest()
        if digest != work["source_text_sha256"]:
            problems.append(f"{work['slug']}: full-text SHA-256 mismatch")
        if digest in seen_hashes:
            problems.append(f"{work['slug']}: duplicate full text")
        seen_hashes.add(digest)
        forbidden = ("project gutenberg", "end of the project gutenberg")
        if any(marker in text.lower() for marker in forbidden):
            problems.append(f"{work['slug']}: Gutenberg boilerplate present")
        if any(line.strip().casefold() in {"contents", "table of contents"} for line in text[:4_000].splitlines()):
            problems.append(f"{work['slug']}: initial contents page present")
        headings = sum(bool(re.match(r"(?i)^\s*(?:(?:chapter|book|part|phase)\b.*|(?:\d+|[ivxlcdm]+)\.\s+\S.*|(?:\d+|[ivxlcdm]+)\.?)\s*$", line)) for line in text.splitlines())
        if headings < max(1, work["expected_chapters"] // 4):
            problems.append(f"{work['slug']}: insufficient headings ({headings})")
        pages = page_count(text)
        if pages < 25:
            problems.append(f"{work['slug']}: only {pages} Bookfin pages")
        total_pages += pages
        author_pages[work["author"]] += pages
        print(f"{work['slug']}: {pages} pages | {headings} headings | {digest}")

    print(f"\nWorks: {len(manifest['works'])}; pages: {total_pages}; authors: {len(author_pages)}")
    for author, pages in sorted(author_pages.items()):
        print(f"{author}: {pages} pages ({pages / total_pages:.2%})")
    if manifest.get("rejections"):
        print(f"Rejections retained in manifest: {len(manifest['rejections'])}")
    if problems:
        print("\nFAILED")
        print("\n".join(problems))
        return 1
    print("\nPASSED: source hashes, duplicates, boilerplate, initial TOCs, structural headings and page floor.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
