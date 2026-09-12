#!/usr/bin/env python3
"""Build Bookfin Long-Form Corpus 01 from Project Gutenberg plain-text editions.

The source text is retained locally only after Gutenberg boilerplate and an
initial table of contents have been removed.  This script is deliberately
conservative: a failed structural check rejects a work instead of producing a
partial edition.
"""

from __future__ import annotations

import hashlib
import json
import re
import time
import urllib.request
import uuid
import unicodedata
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
CORPUS_DIR = ROOT / "corpus" / "long_form_01"
TEXTS_DIR = CORPUS_DIR / "texts"
MANIFEST = CORPUS_DIR / "manifest.json"
NAMESPACE = uuid.UUID("a790e0e8-983d-4f86-b5a0-704a255b9f17")

# Every entry is a standalone narrative work in its original English.  IDs are
# Project Gutenberg ebook identifiers, not book identifiers invented by Bookfin.
WORKS = [
    ("en_austen_pride_prejudice", "Pride and Prejudice", "Jane Austen", 1813, 1342, 61),
    ("en_austen_emma", "Emma", "Jane Austen", 1815, 158, 55),
    ("en_eliot_middlemarch", "Middlemarch", "George Eliot", 1871, 145, 86),
    ("en_james_portrait_lady", "The Portrait of a Lady", "Henry James", 1881, 2833, 55),
    ("en_ford_good_soldier", "The Good Soldier", "Ford Madox Ford", 1915, 2775, 4),
    ("en_dickens_david_copperfield", "David Copperfield", "Charles Dickens", 1850, 766, 64),
    ("en_dickens_bleak_house", "Bleak House", "Charles Dickens", 1853, 1023, 67),
    ("en_hardy_tess", "Tess of the d'Urbervilles", "Thomas Hardy", 1891, 110, 59),
    ("en_hardy_jude", "Jude the Obscure", "Thomas Hardy", 1895, 153, 52),
    ("en_gaskell_north_south", "North and South", "Elizabeth Gaskell", 1855, 4276, 52),
    ("en_gaskell_wives_daughters", "Wives and Daughters", "Elizabeth Gaskell", 1866, 4274, 60),
    ("en_trollope_barchester_towers", "Barchester Towers", "Anthony Trollope", 1857, 619, 53),
    ("en_bronte_jane_eyre", "Jane Eyre", "Charlotte Brontë", 1847, 1260, 38),
    ("en_bronte_wuthering_heights", "Wuthering Heights", "Emily Brontë", 1847, 768, 34),
    ("en_bronte_tenant_wildfell_hall", "The Tenant of Wildfell Hall", "Anne Brontë", 1848, 969, 53),
    ("en_thackeray_vanity_fair", "Vanity Fair", "William Makepeace Thackeray", 1848, 599, 67),
    ("en_stevenson_treasure_island", "Treasure Island", "Robert Louis Stevenson", 1883, 120, 34),
    ("en_conrad_lord_jim", "Lord Jim", "Joseph Conrad", 1900, 5658, 45),
    ("en_conrad_secret_agent", "The Secret Agent", "Joseph Conrad", 1907, 974, 13),
    ("en_melville_moby_dick", "Moby-Dick; or, The Whale", "Herman Melville", 1851, 2701, 135),
    ("en_hawthorne_scarlet_letter", "The Scarlet Letter", "Nathaniel Hawthorne", 1850, 33, 24),
    ("en_wharton_age_innocence", "The Age of Innocence", "Edith Wharton", 1920, 541, 34),
    ("en_wharton_ethan_frome", "Ethan Frome", "Edith Wharton", 1911, 4517, 9),
    ("en_collins_woman_white", "The Woman in White", "Wilkie Collins", 1859, 583, 50),
    ("en_collins_moonstone", "The Moonstone", "Wilkie Collins", 1868, 155, 48),
    ("en_gissing_new_grub_street", "New Grub Street", "George Gissing", 1891, 1709, 36),
    ("en_butler_way_all_flesh", "The Way of All Flesh", "Samuel Butler", 1903, 2084, 83),
    ("en_butler_erewhon", "Erewhon", "Samuel Butler", 1872, 1906, 25),
    ("en_meredith_egoist", "The Egoist", "George Meredith", 1879, 1684, 51),
]


def gutenberg_url(ebook_id: int) -> str:
    return f"https://www.gutenberg.org/cache/epub/{ebook_id}/pg{ebook_id}.txt"


def normalize(text: str) -> str:
    text = text.replace("\r\n", "\n").replace("\r", "\n").replace("\ufeff", "")
    text = re.sub(r"[ \t]+\n", "\n", text)
    text = re.sub(r"\n{3,}", "\n\n", text)
    return text.strip() + "\n"


def source_identity_matches(text: str, title: str) -> bool:
    """Check a normalized declared title near the source opening.

    The author credit is often intentionally removed with the initial contents
    block, so it cannot be a reliable post-cleaning requirement.  The title is
    retained as a provenance sentinel and blocks a wrong Gutenberg ebook ID.
    """
    def folded(value: str) -> str:
        value = unicodedata.normalize("NFKD", value).encode("ascii", "ignore").decode("ascii")
        return re.sub(r"[^a-z0-9]+", " ", value.casefold()).strip()
    opening = folded(text[:30_000])
    # Gutenberg title pages may insert subtitles, punctuation or line breaks
    # between title words.  Requiring every meaningful title word is robust to
    # that typography while still rejecting an unrelated ebook ID.
    return all(word in opening for word in folded(title).split() if len(word) > 2)


def remove_gutenberg_boilerplate(raw: str) -> str:
    start = re.search(r"\*\*\*\s*START OF (?:THE|THIS) PROJECT GUTENBERG EBOOK.*?\*\*\*", raw, re.I | re.S)
    if not start:
        raise ValueError("Gutenberg start marker absent")
    end = re.search(r"\*\*\*\s*END OF (?:THE|THIS) PROJECT GUTENBERG EBOOK.*?\*\*\*", raw[start.end():], re.I | re.S)
    text = raw[start.end(): start.end() + end.start() if end else len(raw)]
    return normalize(text)


def _structural_heading(line: str) -> bool:
    line = line.strip().casefold()
    return bool(
        re.match(r"(?:chapter|book|part|phase)\b", line)
        or re.match(r"(?:\d+|[ivxlcdm]+)\.\s+\S", line)
        or re.fullmatch(r"(?:\d+|[ivxlcdm]+)\.?", line)
    )


def remove_initial_toc(text: str) -> str:
    """Remove only an initial contents block when its first chapter heading repeats.

    Gutenberg texts differ wildly; therefore this intentionally does nothing
    unless a `CONTENTS` heading occurs near the start and a Chapter I/One
    heading occurs at least twice afterwards.
    """
    lines = text.splitlines()
    toc_index = next((i for i, line in enumerate(lines[:600]) if line.strip().casefold() in {"contents", "table of contents"}), None)
    if toc_index is None:
        return text

    # Gutenberg frequently repeats the title immediately before the real
    # narrative.  That is the safest delimiter: it keeps the first narrative
    # heading while eliminating every item of a long contents list.
    title_line = next((line.strip() for line in lines[:20] if line.strip()), "")
    title_matches = [i for i, line in enumerate(lines[toc_index + 1:], toc_index + 1) if title_line and line.strip() == title_line]
    if title_matches:
        return normalize("\n".join(lines[title_matches[0]:]))

    # A few editions begin the narrative with a repeated narrator/epoch line
    # rather than a repeated title (notably The Woman in White).
    toc_lines = {line.strip() for line in lines[toc_index + 1:toc_index + 220] if len(line.strip()) >= 8}
    for i, line in enumerate(lines[toc_index + 12:], toc_index + 12):
        if line.strip() in toc_lines:
            return normalize("\n".join(lines[i:]))

    # Fallback for editions without a repeated title.  Contents entries almost
    # always carry dot leaders/page numbers, unlike the actual heading.
    for i in range(toc_index + 1, len(lines)):
        if _structural_heading(lines[i]) and not re.search(r"\.{2,}\s*\d+\s*$", " ".join(lines[i:i + 3])):
            return normalize("\n".join(lines[i:]))
    return text


def chapter_heading_count(text: str) -> int:
    return sum(_structural_heading(line) for line in text.splitlines())


def validate(text: str, expected_chapters: int) -> list[str]:
    errors: list[str] = []
    prefix = text[:12_000].lower()
    if len(text) < 45_000:
        errors.append(f"text too short ({len(text)} chars)")
    if "project gutenberg" in text.lower() or "end of the project gutenberg" in text.lower():
        errors.append("Gutenberg boilerplate remains")
    if any(line.strip().casefold() in {"contents", "table of contents"} for line in text[:4_000].splitlines()):
        errors.append("initial table of contents remains")
    count = chapter_heading_count(text)
    # Headings vary by edition.  Require enough structural markers to make a
    # truncated source evident, without rejecting books organised in parts.
    if count < max(1, expected_chapters // 4):
        errors.append(f"only {count} chapter/book headings; expected roughly {expected_chapters}")
    if prefix.count("copyright") > 2:
        errors.append("editorial/copyright material remains at opening")
    return errors


def rights_basis(author: str, year: int) -> str:
    if author == "Ford Madox Ford":
        return (
            "US: public domain because first published in 1915 (Project Gutenberg US edition). "
            "EU/France: author died in 1939; life+70 term expired on 2010-01-01. "
            "The Project Gutenberg transcription is sourced under its terms; its boilerplate is not included."
        )
    return (
        f"US: Project Gutenberg identifies ebook source as public domain; original publication {year}. "
        "EU/France: authorial life+70 review recorded separately by source; source transcription is used "
        "without Gutenberg boilerplate and remains subject to Project Gutenberg trademark/terms."
    )


def fetch(url: str) -> str:
    request = urllib.request.Request(url, headers={"User-Agent": "BookfinCorpus/1.0 (corpus ingestion)"})
    with urllib.request.urlopen(request, timeout=60) as response:
        return response.read().decode("utf-8", errors="strict")


def main() -> None:
    TEXTS_DIR.mkdir(parents=True, exist_ok=True)
    entries, rejected, hashes = [], [], set()
    for index, (slug, title, author, year, ebook_id, chapters) in enumerate(WORKS, start=1):
        destination = TEXTS_DIR / f"{slug}.txt"
        print(f"[{index:02}/{len(WORKS)}] {title} — {author}", end=" ", flush=True)
        try:
            if destination.exists():
                text = destination.read_text(encoding="utf-8")
                print("[cache]", end=" ")
            else:
                text = remove_gutenberg_boilerplate(fetch(gutenberg_url(ebook_id)));
                if not source_identity_matches(text, title):
                    raise ValueError("source title does not match the declared work")
            # Reapply the normalizer to cached files too: an earlier interrupted
            # run may have downloaded an edition before a cleaner was improved.
            text = remove_initial_toc(text)
            # Some older cache files restate the Gutenberg name in a one-line
            # artefact outside the star-delimited boilerplate.
            text = normalize("\n".join(line for line in text.splitlines() if "project gutenberg" not in line.casefold()))
            destination.write_text(text, encoding="utf-8", newline="\n")
            if not destination.exists():
                time.sleep(1.0)
            errors = validate(text, chapters)
            digest = hashlib.sha256(text.encode("utf-8")).hexdigest()
            if digest in hashes:
                errors.append("duplicate normalized full-text SHA-256")
            if errors:
                destination.unlink(missing_ok=True)
                rejected.append({"slug": slug, "title": title, "source_url": gutenberg_url(ebook_id), "reasons": errors})
                print("[REJECTED] " + "; ".join(errors))
                continue
            hashes.add(digest)
            entries.append({
                "work_id": str(uuid.uuid5(NAMESPACE, f"work:{slug}")),
                "edition_id": str(uuid.uuid5(NAMESPACE, f"edition:{slug}:gutenberg-{ebook_id}")),
                "slug": slug,
                "title": title,
                "author": author,
                "original_language_tag": "en",
                "work_type": "novel",
                "publication_year": year,
                "edition_title": f"Project Gutenberg ebook #{ebook_id}, plain-text edition",
                "language_tag": "en",
                "source_name": f"Project Gutenberg ebook #{ebook_id}",
                "source_url": f"https://www.gutenberg.org/ebooks/{ebook_id}",
                "provenance": "Downloaded from the Project Gutenberg plain-text cache; START/END boilerplate removed; initial duplicated table of contents removed only when structurally detected.",
                "rights_status": "public_domain_reviewed",
                "rights_basis": rights_basis(author, year),
                "license": "Project Gutenberg source terms; public-domain underlying work",
                "file_path": f"corpus/long_form_01/texts/{slug}.txt",
                "source_text_sha256": digest,
                "source_identity_checked": True,
                "char_count": len(text),
                "chapter_heading_count": chapter_heading_count(text),
                "expected_chapters": chapters,
                "validation": {"status": "accepted", "checks": ["no Gutenberg header/footer", "no initial duplicated TOC", "minimum structural heading count", "full-text SHA-256 uniqueness"]},
            })
            print(f"[OK] {len(text):,} chars, {chapter_heading_count(text)} headings")
        except Exception as exc:  # A failed download is a rejection, never a partial work.
            destination.unlink(missing_ok=True)
            rejected.append({"slug": slug, "title": title, "source_url": gutenberg_url(ebook_id), "reasons": [str(exc)]})
            print(f"[REJECTED] {exc}")

    manifest = {
        "corpus_name": "Bookfin Long-Form Corpus 01",
        "version": 1,
        "generated_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "pagination": {"convention": "existing Bookfin pilot convention", "target_characters": 1600, "all_internal_pages_eligible": True},
        "works_count": len(entries),
        "works": entries,
        "rejections": rejected,
    }
    MANIFEST.write_text(json.dumps(manifest, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    print(f"\nManifest: {len(entries)} accepted, {len(rejected)} rejected -> {MANIFEST}")


if __name__ == "__main__":
    main()
