"""Build the isolated three-source Corpus V2 proof of concept.

This is intentionally not an importer. It reads the archived structured source
files, writes only ``corpus/curation/poc_output``, and never contacts a service
or alters the V1 corpus.
"""

from __future__ import annotations

import json
import hashlib
from pathlib import Path

from corpus_v2 import document_hash, html_to_document, paginate_document


ROOT = Path(__file__).resolve().parents[1]
SOURCE_DIR = ROOT / "corpus" / "curation" / "poc_sources"
OUTPUT_DIR = ROOT / "corpus" / "curation" / "poc_output"

SOURCES = (
    {
        "id": "en_austen_pride_prejudice",
        "path": "en_austen_pride_prejudice_pg42671.html",
        "source_kind": "project_gutenberg_html",
        "content_root_id": None,
        "exclude_ids": {"pg-header", "pg-footer"},
        # An explicit, reviewed reading-start marker for this archived edition;
        # it avoids title-page illustrations and Gutenberg administrative text.
        "start_marker": "<h2>PRIDE &amp; PREJUDICE.</h2>",
    },
    {
        "id": "fr_maupassant_la_parure",
        "path": "fr_maupassant_la_parure_wikisource.html",
        "source_kind": "wikisource_html",
        "content_root_id": "mw-content-text",
        "exclude_ids": set(),
        "start_marker": None,
    },
    {
        "id": "es_quiroga_a_la_deriva",
        "path": "es_quiroga_a_la_deriva_wikisource.html",
        "source_kind": "wikisource_html",
        "content_root_id": "mw-content-text",
        "exclude_ids": set(),
        "start_marker": None,
    },
)


def preview(page: dict[str, object], limit: int = 360) -> str:
    chunks: list[str] = []
    for block in page["blocks"]:  # type: ignore[index]
        for span in block.get("spans", []):  # type: ignore[union-attr]
            chunks.append(span["text"])
    return "".join(chunks).replace("\n", " ")[:limit]


def main() -> None:
    OUTPUT_DIR.mkdir(parents=True, exist_ok=True)
    index: list[dict[str, object]] = []
    for source in SOURCES:
        source_path = SOURCE_DIR / source["path"]
        source_bytes = source_path.read_bytes()
        html = source_bytes.decode("utf-8")
        if source["start_marker"]:
            marker = source["start_marker"]
            if marker not in html:
                raise ValueError(f"Reviewed start marker missing: {source['id']}")
            html = html[html.index(marker) :]
        document = html_to_document(
            html,
            content_root_id=source["content_root_id"],
            exclude_ids=source["exclude_ids"],
        )
        pages = paginate_document(document)
        payload = {
            "poc_only": True,
            "source_kind": source["source_kind"],
            "document_sha256": document_hash(document),
            "document": document,
            "pages": pages,
        }
        destination = OUTPUT_DIR / f"{source['id']}.json"
        destination.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        index.append(
            {
                "id": source["id"],
                "source_archive": str(source_path),
                "source_sha256": hashlib.sha256(source_bytes).hexdigest(),
                "output": str(destination),
                "blocks": len(document["blocks"]),
                "pages": len(pages),
                "first_page_preview": preview(pages[0]) if pages else "",
            }
        )
    (OUTPUT_DIR / "index.json").write_text(
        json.dumps({"poc_only": True, "items": index}, ensure_ascii=False, indent=2) + "\n",
        encoding="utf-8",
    )


if __name__ == "__main__":
    main()
