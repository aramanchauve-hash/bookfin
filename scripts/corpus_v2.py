"""Bookfin Corpus V2 normalisation and block-safe pilot pagination.

The module is dependency-free. It accepts retained HTML (preferred) or a
legacy plain-text source (lossy fallback). It does not touch V1 database pages.
"""

from __future__ import annotations

import hashlib
import json
import re
from html.parser import HTMLParser
from typing import Any, Iterable

BLOCK_TYPES = {"paragraph", "heading", "scene_break", "blockquote", "verse"}
INLINE_TAGS = {"i": "italic", "em": "italic", "b": "bold", "strong": "bold"}
BLOCK_TAGS = {"p": "paragraph", "blockquote": "blockquote", "pre": "verse"}
VOID_TAGS = {"area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source", "track", "wbr"}


def canonical_json(document: dict[str, Any]) -> str:
    return json.dumps(document, ensure_ascii=False, sort_keys=True, separators=(",", ":"))


def document_hash(document: dict[str, Any]) -> str:
    return hashlib.sha256(canonical_json(document).encode("utf-8")).hexdigest()


def _compact_spans(spans: Iterable[dict[str, Any]]) -> list[dict[str, Any]]:
    result: list[dict[str, Any]] = []
    for span in spans:
        text = re.sub(r"[ \t]+", " ", span["text"])
        if not text:
            continue
        style = {key: value for key, value in span.items() if key != "text" and value}
        if result and {key: value for key, value in result[-1].items() if key != "text"} == style:
            result[-1]["text"] += text
        else:
            result.append({"text": text, **style})
    return result


class RichHtmlToBookfin(HTMLParser):
    def __init__(
        self,
        *,
        content_root_id: str | None = None,
        exclude_ids: set[str] | None = None,
    ) -> None:
        super().__init__(convert_charrefs=True)
        self.blocks: list[dict[str, Any]] = []
        self.active: list[dict[str, Any]] = []
        self.spans: list[dict[str, Any]] = []
        self.inline: list[str] = []
        self.content_root_id = content_root_id
        self.exclude_ids = exclude_ids or set()
        self.capture_depth = 0 if content_root_id else 1
        self.exclude_depth = 0
        self.skip_depth = 0

    def handle_starttag(self, tag: str, attrs: list[tuple[str, str | None]]) -> None:
        tag = tag.lower()
        is_void = tag in VOID_TAGS
        attributes = dict(attrs)
        if self.content_root_id and attributes.get("id") == self.content_root_id:
            self.capture_depth = 1
            return
        if self.capture_depth:
            if not is_void:
                self.capture_depth += 1
        else:
            return
        if self.exclude_depth:
            if not is_void:
                self.exclude_depth += 1
            return
        if attributes.get("id") in self.exclude_ids or "ws-noexport" in (attributes.get("class") or ""):
            self.exclude_depth = 1
            return
        if self.skip_depth:
            if not is_void:
                self.skip_depth += 1
            return
        if tag in {"head", "script", "style", "noscript"}:
            self.skip_depth = 1
            return
        if tag in {"h1", "h2", "h3", "h4", "h5", "h6"}:
            self.finish()
            self.active.append({"type": "heading", "level": int(tag[1])})
        elif tag in BLOCK_TAGS:
            self.finish()
            self.active.append({"type": BLOCK_TAGS[tag]})
        elif tag == "hr":
            self.finish()
            self.blocks.append({"type": "scene_break"})
        elif tag == "br":
            self.spans.append({"text": "\n", **self.style()})
        elif tag in INLINE_TAGS:
            self.inline.append(INLINE_TAGS[tag])

    def handle_endtag(self, tag: str) -> None:
        tag = tag.lower()
        if tag in VOID_TAGS:
            return
        if not self.capture_depth:
            return
        if self.exclude_depth:
            self.exclude_depth -= 1
            self.capture_depth -= 1
            return
        if self.skip_depth:
            self.skip_depth -= 1
            self.capture_depth -= 1
            return
        if tag in {"h1", "h2", "h3", "h4", "h5", "h6", *BLOCK_TAGS}:
            self.finish()
        elif tag in INLINE_TAGS and INLINE_TAGS[tag] in self.inline:
            self.inline.reverse()
            self.inline.remove(INLINE_TAGS[tag])
            self.inline.reverse()
        self.capture_depth -= 1

    def style(self) -> dict[str, bool]:
        return {style: True for style in self.inline}

    def handle_data(self, data: str) -> None:
        if not self.capture_depth or self.exclude_depth or self.skip_depth or not data or not data.strip():
            return
        if not self.active:
            self.active.append({"type": "paragraph"})
        self.spans.append({"text": data, **self.style()})

    def finish(self) -> None:
        if not self.active:
            return
        block = self.active.pop()
        spans = _compact_spans(self.spans)
        self.spans = []
        if spans:
            block["spans"] = spans
            self.blocks.append(block)

    def document(self) -> dict[str, Any]:
        self.finish()
        return {"format": "bookfin.document.v1", "blocks": self.blocks}


def html_to_document(
    html: str,
    *,
    content_root_id: str | None = None,
    exclude_ids: set[str] | None = None,
) -> dict[str, Any]:
    """Normalise retained HTML, optionally limited to one verified content root.

    ``content_root_id`` lets a source adapter retain only a publisher's reading
    surface (for example MediaWiki's ``mw-content-text``).  It is deliberately
    a narrow, explicit choice: no heuristic scraping is used for production.
    """
    parser = RichHtmlToBookfin(
        content_root_id=content_root_id,
        exclude_ids=exclude_ids,
    )
    parser.feed(html)
    parser.close()
    return parser.document()


def text_to_document(text: str) -> dict[str, Any]:
    blocks: list[dict[str, Any]] = []
    for raw in re.split(r"\n\s*\n", text.strip()):
        value = re.sub(r"\s*\n\s*", " ", raw).strip()
        if value:
            blocks.append({"type": "paragraph", "spans": [{"text": value}]})
    return {"format": "bookfin.document.v1", "blocks": blocks}


def block_length(block: dict[str, Any]) -> int:
    return sum(len(span["text"]) for span in block.get("spans", []))


def paginate_document(document: dict[str, Any], target_characters: int = 1600) -> list[dict[str, Any]]:
    """Pages never split a block or span; an oversized paragraph is preserved."""
    if document.get("format") != "bookfin.document.v1":
        raise ValueError("Unsupported Bookfin document format")
    pages: list[dict[str, Any]] = []
    current: list[dict[str, Any]] = []
    current_size = 0
    for block in document.get("blocks", []):
        if block.get("type") not in BLOCK_TYPES:
            raise ValueError("Unsupported block type")
        size = block_length(block)
        if current and current_size + size > target_characters:
            pages.append({"format": "bookfin.page.v1", "blocks": current})
            current, current_size = [], 0
        current.append(block)
        current_size += size
    if current:
        pages.append({"format": "bookfin.page.v1", "blocks": current})
    return pages
