"""Regression audit for the Table-Talk source-to-V2 structure; no corpus writes."""
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "corpus/curated_v1/sources/en/en-hazlitt-table-talk.html"
NORMALIZED = ROOT / "corpus/curated_v1/normalized/en/en-hazlitt-table-talk.json"
PAGES = ROOT / "corpus/curated_v1/pages/en/en-hazlitt-table-talk_pages.json"


def block_text(block):
    return "".join(span["text"] for span in block.get("spans", []))


def main():
    source = SOURCE.read_text(encoding="utf-8")
    assert "kind of shadowy abstraction,\n    </p>\n<pre>\n Pure in the last recesses" in source

    document = json.loads(NORMALIZED.read_text(encoding="utf-8"))["document"]["blocks"]
    index = next(i for i, block in enumerate(document) if "shadowy abstraction" in block_text(block))
    assert [document[index + offset]["type"] for offset in range(3)] == ["paragraph", "verse", "paragraph"]
    assert "Pure in the last recesses of the mind" in block_text(document[index + 1])

    pages = json.loads(PAGES.read_text(encoding="utf-8"))["pages"]
    page = next(page for page in pages if any("Pure in the last recesses" in block_text(block) for block in page["blocks"]))
    types = [block["type"] for block in page["blocks"]]
    verse_index = types.index("verse")
    assert types[verse_index - 1:verse_index + 2] == ["paragraph", "verse", "paragraph"]


if __name__ == "__main__":
    main()
