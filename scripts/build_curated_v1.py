"""Bookfin Curated V1 Corpus Builder.

Acquires, normalizes, paginates, validates, and reports on the 73 ACCEPTED works.
Strictly local: ZERO writes to Railway, zero DB modifications, zero EAS builds.
"""

from __future__ import annotations

import hashlib
import json
import re
import time
import urllib.request
import urllib.error
from datetime import datetime, timezone
from html.parser import HTMLParser
from pathlib import Path
from typing import Any, Iterable

ROOT = Path(__file__).resolve().parents[1]
CURATED_DIR = ROOT / "corpus" / "curated_v1"
SOURCES_DIR = CURATED_DIR / "sources"
NORMALIZED_DIR = CURATED_DIR / "normalized"
PAGES_DIR = CURATED_DIR / "pages"
SAMPLES_DIR = CURATED_DIR / "samples"
POC_SOURCES_DIR = ROOT / "corpus" / "curation" / "poc_sources"
CANDIDATES_PATH = ROOT / "corpus" / "curation" / "library_v1_candidates.json"
LOCK_PATH = CURATED_DIR / "curated_v1.lock.json"

HTTP_HEADERS = {
    "User-Agent": "BookfinCuratedV1/1.0 (https://bookfin.app; contact@bookfin.app) Python/3.12"
}

# =============================================================================
# 1. URL RESOLVER FOR 73 ACCEPTED WORKS
# =============================================================================
SPECIAL_URL_OVERRIDES = {
    "en-browne-religio-medici": {
        "url": "https://www.gutenberg.org/cache/epub/586/pg586-images.html",
        "provider": "Project Gutenberg",
        "format": "gutenberg_html",
        "rationale": "Gutenberg XHTML master #586 (section Religio Medici).",
        "slice_start": "RELIGIO MEDICI.",
        "slice_start_offset": 25000,
        "slice_end": "HYDRIOTAPHIA.",
    },
    "en-browne-hydriotaphia": {
        "url": "https://www.gutenberg.org/cache/epub/586/pg586-images.html",
        "provider": "Project Gutenberg",
        "format": "gutenberg_html",
        "rationale": "Gutenberg XHTML master #586 (section Hydriotaphia, Urn Burial).",
        "slice_start": "HYDRIOTAPHIA.",
        "slice_start_offset": 200000,
        "slice_end": "A LETTER TO A FRIEND",
    },
    "en-doyle-scandal-bohemia": {
        "url": "https://www.gutenberg.org/cache/epub/1661/pg1661-images.html",
        "provider": "Project Gutenberg",
        "format": "gutenberg_html",
        "rationale": "Gutenberg XHTML master #1661 (nouvelle autonome I. A Scandal in Bohemia).",
        "slice_start": "A SCANDAL IN BOHEMIA",
        "slice_start_offset": 5000,
        "slice_end": "THE RED-HEADED LEAGUE",
    },
    "en-doyle-red-headed-league": {
        "url": "https://www.gutenberg.org/cache/epub/1661/pg1661-images.html",
        "provider": "Project Gutenberg",
        "format": "gutenberg_html",
        "rationale": "Gutenberg XHTML master #1661 (nouvelle autonome II. The Red-Headed League).",
        "slice_start": "THE RED-HEADED LEAGUE",
        "slice_start_offset": 50000,
        "slice_end": "A CASE OF IDENTITY",
    },
    "fr-mirbeau-journal-femme-chambre": {
        "url": "https://www.gutenberg.org/cache/epub/43170/pg43170-images.html",
        "provider": "Project Gutenberg",
        "format": "gutenberg_html",
        "rationale": "Gutenberg XHTML master #43170 (roman complet).",
    },
    "fr-maupassant-bel-ami": {
        "url": "https://fr.wikisource.org/wiki/Bel-Ami/%C3%89dition_Ollendorff,_1901/Texte_entier",
        "provider": "Wikisource",
        "format": "wikisource_html",
        "rationale": "Wikisource HTML complet édition Ollendorff 1901 (Texte entier).",
    },
    "fr-maupassant-pierre-et-jean": {
        "url": "https://fr.wikisource.org/wiki/Pierre_et_Jean/Texte_entier",
        "provider": "Wikisource",
        "format": "wikisource_html",
        "rationale": "Wikisource HTML complet édition Ollendorff 1888 (Texte entier).",
    },
    "fr-maupassant-boule-de-suif": {
        "url": "https://www.gutenberg.org/cache/epub/10746/pg10746-images.html",
        "provider": "Project Gutenberg",
        "format": "gutenberg_html",
        "rationale": "Gutenberg XHTML master #10746 (Boule de Suif texte intégral).",
    },
    "fr-schwob-vies-imaginaires": {
        "url": "https://fr.wikisource.org/wiki/Vies_imaginaires/Texte_entier",
        "provider": "Wikisource",
        "format": "wikisource_html",
        "rationale": "Wikisource HTML complet édition Charpentier 1896 (Texte entier).",
    },
    "fr-schwob-livre-de-monelle": {
        "url": "https://fr.wikisource.org/wiki/La_Lampe_de_Psych%C3%A9/Le_Livre_de_Monelle",
        "provider": "Wikisource",
        "format": "wikisource_html",
        "rationale": "Wikisource HTML complet issu de La Lampe de Psyché (Mercure de France 1894).",
    },
    "fr-lesage-diable-boiteux": {
        "url": "https://www.gutenberg.org/cache/epub/33434/pg33434-images.html",
        "provider": "Project Gutenberg",
        "format": "gutenberg_html",
        "rationale": "Gutenberg XHTML master #33434 (roman complet).",
    },
    "fr-merimee-carmen": {
        "url": "https://www.gutenberg.org/cache/epub/2465/pg2465-images.html",
        "provider": "Project Gutenberg",
        "format": "gutenberg_html",
        "rationale": "Gutenberg XHTML master #2465 (Carmen texte intégral).",
    },
    "fr-diderot-neveu-de-rameau": {
        "url": "https://www.gutenberg.org/cache/epub/13862/pg13862-images.html",
        "provider": "Project Gutenberg",
        "format": "gutenberg_html",
        "rationale": "Gutenberg XHTML master #13862 (Le Neveu de Rameau texte intégral).",
    },
    "fr-nerval-aurelia": {
        "url": "https://fr.wikisource.org/wiki/Aur%C3%A9lia/Texte_entier",
        "provider": "Wikisource",
        "format": "wikisource_html",
        "rationale": "Wikisource HTML complet (Texte entier).",
    },
    "fr-barbey-les-diaboliques": {
        "url": "https://www.gutenberg.org/cache/epub/13848/pg13848-images.html",
        "provider": "Project Gutenberg",
        "format": "gutenberg_html",
        "rationale": "Gutenberg XHTML master #13848 (recueil complet).",
    },
    "fr-huysmans-a-rebours": {
        "url": "https://fr.wikisource.org/wiki/%C3%80_rebours/Texte_entier",
        "provider": "Wikisource",
        "format": "wikisource_html",
        "rationale": "Wikisource HTML complet (Texte entier).",
    },
    "fr-bloy-histoires-desobligeantes": {
        "url": "https://fr.wikisource.org/wiki/Histoires_d%C3%A9sobligeantes/Texte_entier",
        "provider": "Wikisource",
        "format": "wikisource_html",
        "rationale": "Wikisource HTML complet (Texte entier).",
    },
    "fr-stendhal-chartreuse-de-parme": {
        "url": "https://www.gutenberg.org/cache/epub/796/pg796-images.html",
        "provider": "Project Gutenberg",
        "format": "gutenberg_html",
        "rationale": "Gutenberg XHTML master #796 (roman complet).",
    },
    "fr-stendhal-chroniques-italiennes": {
        "urls": [
            "https://fr.wikisource.org/wiki/Chroniques_italiennes_(%C3%A9dition_L%C3%A9vy,_1855)/L%E2%80%99Abbesse_de_Castro",
            "https://fr.wikisource.org/wiki/Chroniques_italiennes_(%C3%A9dition_L%C3%A9vy,_1855)/Vittoria_Accoramboni",
            "https://fr.wikisource.org/wiki/Chroniques_italiennes_(%C3%A9dition_L%C3%A9vy,_1855)/Les_Cenci",
            "https://fr.wikisource.org/wiki/Chroniques_italiennes_(%C3%A9dition_L%C3%A9vy,_1855)/La_Duchesse_de_Palliano",
            "https://fr.wikisource.org/wiki/Chroniques_italiennes_(%C3%A9dition_L%C3%A9vy,_1855)/Vanina_Vanini",
        ],
        "url": "https://fr.wikisource.org/wiki/Chroniques_italiennes_(%C3%A9dition_L%C3%A9vy,_1855)",
        "provider": "Wikisource",
        "format": "wikisource_html",
        "rationale": "Wikisource HTML complet des 5 chroniques (édition Lévy 1855 intégrale).",
    },
    "es-cervantes-quijote": {
        "url": "https://www.gutenberg.org/cache/epub/2000/pg2000-images.html",
        "provider": "Project Gutenberg",
        "format": "gutenberg_html",
        "rationale": "Gutenberg XHTML master #2000 (Don Quijote texte intégral).",
    },
    "es-cervantes-la-gitanilla": {
        "url": "https://www.gutenberg.org/cache/epub/61202/pg61202-images.html",
        "provider": "Project Gutenberg",
        "format": "gutenberg_html",
        "rationale": "Gutenberg XHTML master #61202 (nouvelle autonome La gitanilla).",
        "slice_start": "LA JITANILLA.",
        "slice_start_offset": 20000,
        "slice_end": "EL AMANTE LIBERAL.",
    },
    "es-cervantes-rinconete-y-cortadillo": {
        "url": "https://www.gutenberg.org/cache/epub/61202/pg61202-images.html",
        "provider": "Project Gutenberg",
        "format": "gutenberg_html",
        "rationale": "Gutenberg XHTML master #61202 (nouvelle autonome Rinconete y Cortadillo).",
        "slice_start": "RINCONETE Y CORTADILLO.",
        "slice_start_offset": 250000,
        "slice_end": "LA ESPA",
    },
    "es-cervantes-el-licenciado-vidriera": {
        "url": "https://www.gutenberg.org/cache/epub/61202/pg61202-images.html",
        "provider": "Project Gutenberg",
        "format": "gutenberg_html",
        "rationale": "Gutenberg XHTML master #61202 (nouvelle autonome El licenciado Vidriera).",
        "slice_start": "EL LICENCIADO VIDRIERA.",
        "slice_start_offset": 400000,
        "slice_end": "LA FUERZA DE LA SANGRE.",
    },
    "es-cervantes-el-celoso-extremeno": {
        "url": "https://www.gutenberg.org/cache/epub/61202/pg61202-images.html",
        "provider": "Project Gutenberg",
        "format": "gutenberg_html",
        "rationale": "Gutenberg XHTML master #61202 (nouvelle autonome El celoso extremeño).",
        "slice_start": "EL CELOSO ESTREME",
        "slice_start_offset": 500000,
        "slice_end": "LA ILUSTRE FREGONA.",
    },
    "es-cervantes-el-coloquio-de-los-perros": {
        "url": "https://www.gutenberg.org/cache/epub/61202/pg61202-images.html",
        "provider": "Project Gutenberg",
        "format": "gutenberg_html",
        "rationale": "Gutenberg XHTML master #61202 (nouvelle autonome Coloquio de los perros).",
        "slice_start": "COLOQUIO QUE PASO ENTRE CIPION",
        "slice_start_offset": 900000,
        "slice_end": "LA TIA FINGIDA.",
    },
    "es-quevedo-los-suenos": {
        "url": "https://www.gutenberg.org/cache/epub/65999/pg65999-images.html",
        "provider": "Project Gutenberg",
        "format": "gutenberg_html",
        "rationale": "Gutenberg XHTML master #65999 (Los Sueños texte original espagnol).",
    },
    "es-quevedo-el-buscon": {
        "url": "https://www.gutenberg.org/cache/epub/32315/pg32315-images.html",
        "provider": "Project Gutenberg",
        "format": "gutenberg_html",
        "rationale": "Gutenberg XHTML master #32315 (El Buscón texte original espagnol).",
    },
    "es-pardo-bazan-los-pazos-de-ulloa": {
        "url": "https://www.gutenberg.org/cache/epub/18005/pg18005-images.html",
        "provider": "Project Gutenberg",
        "format": "gutenberg_html",
        "rationale": "Gutenberg XHTML master #18005 (Los pazos de Ulloa texte intégral).",
    },
    "es-galdos-misericordia": {
        "url": "https://www.gutenberg.org/cache/epub/21831/pg21831-images.html",
        "provider": "Project Gutenberg",
        "format": "gutenberg_html",
        "rationale": "Gutenberg XHTML master #21831 (Misericordia texte intégral).",
    },
    "es-galdos-dona-perfecta": {
        "url": "https://www.gutenberg.org/cache/epub/15725/pg15725-images.html",
        "provider": "Project Gutenberg",
        "format": "gutenberg_html",
        "rationale": "Gutenberg XHTML master #15725 (Doña Perfecta texte intégral).",
    },
    "es-unamuno-niebla": {
        "url": "https://www.gutenberg.org/cache/epub/49836/pg49836-images.html",
        "provider": "Project Gutenberg",
        "format": "gutenberg_html",
        "rationale": "Gutenberg HTML master de l'édition Renacimiento 1914.",
    },
    "es-valle-inclan-sonata-otono": {
        "url": "https://www.gutenberg.org/cache/epub/37537/pg37537-images.html",
        "provider": "Project Gutenberg",
        "format": "gutenberg_html",
        "rationale": "Gutenberg HTML master de la Sonata de otoño (1902).",
    },
    "es-valle-inclan-luces-de-bohemia": {
        "url": "https://www.gutenberg.org/cache/epub/68745/pg68745-images.html",
        "provider": "Project Gutenberg",
        "format": "gutenberg_html",
        "rationale": "Gutenberg XHTML master #68745 (Luces de Bohemia: Esperpento).",
    },
}

POC_REUSE = {
    "en-austen-pride-prejudice": "en_austen_pride_prejudice_pg42671.html",
    "fr-maupassant-la-parure": "fr_maupassant_la_parure_wikisource.html",
    "es-quiroga-a-la-deriva": "es_quiroga_a_la_deriva_wikisource.html",
}


def get_download_spec(candidate: dict[str, Any]) -> dict[str, Any]:
    cid = candidate["id"]
    if cid in SPECIAL_URL_OVERRIDES:
        override = SPECIAL_URL_OVERRIDES[cid]
        return {
            "url": override["url"],
            "urls": override.get("urls"),
            "provider": override["provider"],
            "format": override["format"],
            "rationale": override["rationale"],
            "slice_start": override.get("slice_start"),
            "slice_start_offset": override.get("slice_start_offset", 0),
            "slice_end": override.get("slice_end"),
        }
    
    url = candidate["source_url"]
    provider = candidate["source_provider"]
    
    if "gutenberg.org/ebooks/" in url:
        ebook_id = url.rstrip("/").split("/")[-1]
        return {
            "url": f"https://www.gutenberg.org/cache/epub/{ebook_id}/pg{ebook_id}-images.html",
            "provider": "Project Gutenberg",
            "format": "gutenberg_html",
            "rationale": f"Gutenberg XHTML master #{ebook_id} (fichier source de référence sans conteneur).",
        }
    else:
        return {
            "url": url,
            "provider": provider,
            "format": "wikisource_html",
            "rationale": "Wikisource HTML complet structuré (avec transclusion DjVu le cas échéant).",
        }


# =============================================================================
# 2. CANONICAL HASH & JSON UTILITIES
# =============================================================================
def canonical_json(data: Any) -> str:
    return json.dumps(data, ensure_ascii=False, sort_keys=True, separators=(",", ":"))


def sha256_text(text: str) -> str:
    return hashlib.sha256(text.encode("utf-8")).hexdigest()


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


# =============================================================================
# 3. ROBUST HTML TO BOOKFIN DOCUMENT PARSER
# =============================================================================
VOID_TAGS = {"area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source", "track", "wbr"}
INLINE_TAGS = {"i": "italic", "em": "italic", "b": "bold", "strong": "bold", "small": "small_caps"}
BLOCK_TAGS = {"p": "paragraph", "blockquote": "blockquote", "pre": "verse"}


class BookfinHtmlParser(HTMLParser):
    EXCLUDE_CLASSES = {
        "printfooter", "catlinks", "mw-jump-link", "ws-noexport",
        "mw-editsection", "navbox", "ws-header", "ws-footer",
        "noprint", "mw-empty-elt", "sister-project", "pg-boilerplate",
        "pgheader", "pgfooter", "navigation-not-searchable", "metadata",
        "ws-summary",
        # These are source-documentary containers, not literary text. Keeping
        # the exclusion class-based preserves a following sibling such as
        # <span class="pagenum">p. 14</span>MAX as the canonical "MAX".
        "pagenum", "toc",
    }
    EXCLUDE_IDS = {"pg-header", "pg-footer", "catlinks", "mw-navigation", "toc", "header", "footer"}

    def __init__(self, *, content_root_id: str | None = None) -> None:
        super().__init__(convert_charrefs=True)
        self.blocks: list[dict[str, Any]] = []
        self.active: list[dict[str, Any]] = []
        self.spans: list[dict[str, Any]] = []
        self.inline: list[str] = []
        self.content_root_id = content_root_id
        self.capture_depth = 0 if content_root_id else 1
        self.exclude_depth = 0
        self.skip_depth = 0
        # Gutenberg's Quijote has a classed TOC title followed immediately by
        # an unclassed UL. It is still a documented part of that TOC, rather
        # than literary prose.
        self.skip_next_toc_list = False

    def handle_starttag(self, tag: str, attrs: list[tuple[str, str | None]]) -> None:
        tag = tag.lower()
        is_void = tag in VOID_TAGS
        attributes = dict(attrs)
        classes = {value.casefold() for value in (attributes.get("class") or "").split()}
        elem_id = attributes.get("id")
        is_documentary_contents_table = attributes.get("data-summary", "").casefold() == "cont"
        follows_toc_title = tag in {"ul", "ol"} and self.skip_next_toc_list
        if follows_toc_title:
            self.skip_next_toc_list = False
        if "toc" in classes and tag == "div":
            self.skip_next_toc_list = True

        if self.content_root_id and elem_id == self.content_root_id:
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

        if follows_toc_title or is_documentary_contents_table or elem_id in self.EXCLUDE_IDS or bool(classes & self.EXCLUDE_CLASSES):
            if not is_void:
                self.exclude_depth = 1
            return

        if self.skip_depth:
            if not is_void:
                self.skip_depth += 1
            return

        if tag in {"head", "script", "style", "noscript"}:
            if not is_void:
                self.skip_depth = 1
            return

        if tag in {"h1", "h2", "h3", "h4", "h5", "h6"}:
            self.finish()
            self.active.append({"type": "heading", "level": int(tag[1])})
        elif tag in BLOCK_TAGS:
            self.finish()
            self.active.append({"type": BLOCK_TAGS[tag]})
        elif tag == "li":
            # A list item is an explicit DOM text boundary. Bookfin has no
            # list block type, so retain it as a standalone paragraph.
            self.finish()
            self.active.append({"type": "paragraph"})
        elif tag == "div" and "verse" in classes:
            # Gutenberg marks verse lines with class="verse". Do not join
            # adjacent lines into a false prose word boundary.
            self.finish()
            self.active.append({"type": "verse"})
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
            if self.content_root_id is not None:
                self.capture_depth -= 1
            return
        if self.skip_depth:
            self.skip_depth -= 1
            if self.content_root_id is not None:
                self.capture_depth -= 1
            return
        if tag in {"h1", "h2", "h3", "h4", "h5", "h6", "li", *BLOCK_TAGS}:
            self.finish()
        elif tag == "div" and self.active and self.active[-1].get("type") == "verse":
            self.finish()
        elif tag in INLINE_TAGS and INLINE_TAGS[tag] in self.inline:
            self.inline.reverse()
            self.inline.remove(INLINE_TAGS[tag])
            self.inline.reverse()
        if self.content_root_id is not None:
            self.capture_depth -= 1

    def style(self) -> dict[str, bool]:
        return {s: True for s in self.inline}

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
        spans = self.compact_spans(self.spans)
        self.spans = []
        if spans:
            full_text = "".join(s["text"] for s in spans).strip()
            # Omit empty whitespace blocks
            if full_text:
                block["spans"] = spans
                self.blocks.append(block)

    @staticmethod
    def compact_spans(spans: Iterable[dict[str, Any]]) -> list[dict[str, Any]]:
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

    def document(self) -> dict[str, Any]:
        self.finish()
        return {"format": "bookfin.document.v1", "blocks": self.blocks}


def normalize_html_to_document(html: str, *, source_format: str) -> dict[str, Any]:
    content_root = "mw-content-text" if source_format == "wikisource_html" else None
    parser = BookfinHtmlParser(content_root_id=content_root)
    parser.feed(html)
    parser.close()
    
    clean_blocks = []
    for b in parser.document()["blocks"]:
        b_txt = "".join(s.get("text", "") for s in b.get("spans", []))
        if any(marker in b_txt for marker in [
            "START OF THE PROJECT GUTENBERG EBOOK",
            "END OF THE PROJECT GUTENBERG EBOOK",
            "Project Gutenberg License",
            "THE FULL PROJECT GUTENBERG LICENSE",
            "Project Gutenberg™ License",
            "*** START OF THE PROJECT",
            "*** END OF THE PROJECT",
            "Section 1. General Terms of Use",
            "Section 2. Information about the Mission",
            "Section 3. Information about the Project Gutenberg",
            "Section 5. General Information About Project Gutenberg",
            "The Project Gutenberg eBook",
        ]):
            continue
        clean_blocks.append(b)
        
    return {"format": "bookfin.document.v1", "blocks": clean_blocks}


# =============================================================================
# 4. DETERMINISTIC PAGINATION WITH CLEAN SPAN PRESERVATION
# =============================================================================
def block_length(block: dict[str, Any]) -> int:
    return sum(len(s.get("text", "")) for s in block.get("spans", []))


def split_spans_at(spans: list[dict[str, Any]], split_pos: int) -> tuple[list[dict[str, Any]], list[dict[str, Any]]]:
    left: list[dict[str, Any]] = []
    right: list[dict[str, Any]] = []
    cur = 0
    for s in spans:
        txt = s.get("text", "")
        slen = len(txt)
        style = {k: v for k, v in s.items() if k != "text"}
        
        if cur + slen <= split_pos:
            left.append(s)
        elif cur >= split_pos:
            right.append(s)
        else:
            cut = split_pos - cur
            l_txt = txt[:cut]
            r_txt = txt[cut:]
            if l_txt:
                left.append({"text": l_txt, **style})
            if r_txt:
                right.append({"text": r_txt, **style})
        cur += slen
    return left, right


def find_split_point(text: str, target: int, window: int = 150) -> int:
    min_b = max(10, target - window)
    max_b = min(len(text) - 1, target + window)
    
    # 1. Look for sentence ending (. ! ? » ”) followed by whitespace
    best_sent, best_sent_dist = -1, 999999
    for i in range(min_b, max_b):
        if text[i] in ".!?»”" and (i + 1 == len(text) or text[i+1] in " \n\r\t"):
            dist = abs((i + 1) - target)
            if dist < best_sent_dist:
                best_sent_dist = dist
                best_sent = i + 1
    if best_sent != -1:
        return best_sent
    
    # 2. Look for word boundary (whitespace)
    best_word, best_word_dist = -1, 999999
    for i in range(min_b, max_b):
        if text[i] in " \t\n":
            dist = abs(i - target)
            if dist < best_word_dist:
                best_word_dist = dist
                best_word = i
    if best_word != -1:
        return best_word
    return target


def paginate_document(
    document: dict[str, Any],
    *,
    work_id: str,
    edition_id: str,
    language: str,
    target_characters: int = 1600,
) -> list[dict[str, Any]]:
    pages: list[dict[str, Any]] = []
    current_blocks: list[dict[str, Any]] = []
    current_chars = 0
    page_num = 1
    
    blocks = list(document.get("blocks", []))
    i = 0
    while i < len(blocks):
        b = blocks[i]
        b_len = block_length(b)
        b_type = b.get("type", "paragraph")
        
        # Avoid orphan headings at bottom of page if already >= 1000 chars
        if b_type == "heading" and current_chars >= 1000:
            if current_blocks:
                page_blocks = current_blocks
                pages.append({
                    "format": "bookfin.page.v1",
                    "work_id": work_id,
                    "edition_id": edition_id,
                    "page_sequence_number": page_num,
                    "version": "2.0",
                    "language": language,
                    "content_hash": sha256_text(canonical_json(page_blocks)),
                    "blocks": page_blocks,
                })
                page_num += 1
                current_blocks = []
                current_chars = 0
        
        # Block fits comfortably within target margin
        if current_chars + b_len <= target_characters + 180:
            current_blocks.append(b)
            current_chars += b_len
            i += 1
        else:
            # Over target: if already at comfortable size, end page cleanly at block boundary
            if current_chars >= 1150:
                if current_blocks:
                    page_blocks = current_blocks
                    pages.append({
                        "format": "bookfin.page.v1",
                        "work_id": work_id,
                        "edition_id": edition_id,
                        "page_sequence_number": page_num,
                        "version": "2.0",
                        "language": language,
                        "content_hash": sha256_text(canonical_json(page_blocks)),
                        "blocks": page_blocks,
                    })
                    page_num += 1
                    current_blocks = []
                    current_chars = 0
            else:
                # Split this block cleanly if paragraph/blockquote
                if b_type in ("paragraph", "blockquote"):
                    spans = b.get("spans", [])
                    full_text = "".join(s.get("text", "") for s in spans)
                    needed = max(50, target_characters - current_chars)
                    split_idx = find_split_point(full_text, needed)
                    
                    left_spans, right_spans = split_spans_at(spans, split_idx)
                    if left_spans:
                        current_blocks.append({"type": b_type, "spans": left_spans})
                    
                    if current_blocks:
                        page_blocks = current_blocks
                        pages.append({
                            "format": "bookfin.page.v1",
                            "work_id": work_id,
                            "edition_id": edition_id,
                            "page_sequence_number": page_num,
                            "version": "2.0",
                            "language": language,
                            "content_hash": sha256_text(canonical_json(page_blocks)),
                            "blocks": page_blocks,
                        })
                        page_num += 1
                        current_blocks = []
                        current_chars = 0
                    
                    if right_spans:
                        blocks[i] = {"type": b_type, "spans": right_spans}
                    else:
                        i += 1
                else:
                    # Heading or scene break: advance page cleanly
                    if current_blocks:
                        page_blocks = current_blocks
                        pages.append({
                            "format": "bookfin.page.v1",
                            "work_id": work_id,
                            "edition_id": edition_id,
                            "page_sequence_number": page_num,
                            "version": "2.0",
                            "language": language,
                            "content_hash": sha256_text(canonical_json(page_blocks)),
                            "blocks": page_blocks,
                        })
                        page_num += 1
                    current_blocks = [b]
                    current_chars = b_len
                    i += 1

    if current_blocks:
        pages.append({
            "format": "bookfin.page.v1",
            "work_id": work_id,
            "edition_id": edition_id,
            "page_sequence_number": page_num,
            "version": "2.0",
            "language": language,
            "content_hash": sha256_text(canonical_json(current_blocks)),
            "blocks": current_blocks,
        })
    return pages


# =============================================================================
# 5. AUTOMATED VALIDATION SUITE
# =============================================================================
def validate_curated_work(
    candidate: dict[str, Any],
    document: dict[str, Any],
    pages: list[dict[str, Any]],
    source_bytes: bytes,
) -> dict[str, Any]:
    issues: list[str] = []
    cid = candidate["id"]
    
    # 1. Blocks & Pages
    blocks = document.get("blocks", [])
    if not blocks:
        issues.append("CRITICAL: Zero blocks extracted")
    if not pages:
        issues.append("CRITICAL: Zero pages generated")
        
    # 2. Numbering continuous
    for expected_idx, p in enumerate(pages, 1):
        if p["page_sequence_number"] != expected_idx:
            issues.append(f"Page numbering discontinuity at {p['page_sequence_number']} (expected {expected_idx})")
        if not p.get("content_hash"):
            issues.append(f"Missing content_hash on page {expected_idx}")
        if p.get("language") != candidate["original_language"]:
            issues.append(f"Page language mismatch on page {expected_idx}: {p.get('language')}")
        if not p.get("blocks"):
            issues.append(f"Empty page blocks on page {expected_idx}")

    # 3. Text content & checks for boilerplate pollution
    full_text = " ".join("".join(s["text"] for s in b.get("spans", [])) for b in blocks)
    
    # Check for Project Gutenberg noise
    if "Project Gutenberg License" in full_text:
        issues.append("Pollution detected: 'Project Gutenberg License' found in text")
    if "START OF THE PROJECT GUTENBERG EBOOK" in full_text:
        issues.append("Pollution detected: 'START OF THE PROJECT GUTENBERG EBOOK' found in text")
    if "END OF THE PROJECT GUTENBERG EBOOK" in full_text:
        issues.append("Pollution detected: 'END OF THE PROJECT GUTENBERG EBOOK' found in text")
        
    # Check for Wikisource noise
    if "Récupérée de" in full_text or "Retrieved from" in full_text or "Obtenido de" in full_text:
        issues.append("Pollution detected: Wikisource URL footer found in text")
        
    # Check UTF-8 validity
    try:
        full_text.encode("utf-8").decode("utf-8")
    except UnicodeError as e:
        issues.append(f"Unicode error: {e}")
        
    # Anomaly checks
    has_italics = any(any(s.get("italic") for s in b.get("spans", [])) for b in blocks)
    headings_count = sum(1 for b in blocks if b.get("type") == "heading")
    
    # Check for gigantic paragraphs (> 12000 chars without punctuation)
    for b in blocks:
        b_len = block_length(b)
        if b_len > 12000:
            issues.append(f"Anomaly: extremely large block ({b_len} chars)")

    return {
        "work_id": cid,
        "valid": len([i for i in issues if i.startswith("CRITICAL") or "Pollution" in i]) == 0,
        "blocks_count": len(blocks),
        "pages_count": len(pages),
        "total_characters": len(full_text),
        "headings_count": headings_count,
        "has_italics": has_italics,
        "issues": issues,
    }


# =============================================================================
# 6. SAMPLES & PREVIEWS FOR HUMAN CONTROL
# =============================================================================
def text_preview(page: dict[str, Any], max_len: int = 350) -> str:
    chunks = []
    for b in page.get("blocks", []):
        for s in b.get("spans", []):
            chunks.append(s.get("text", ""))
    raw = "".join(chunks).replace("\r\n", " ").replace("\n", " ").strip()
    raw = re.sub(r"\s+", " ", raw)
    if len(raw) > max_len:
        return raw[:max_len] + "..."
    return raw


def generate_control_samples(candidate: dict[str, Any], pages: list[dict[str, Any]]) -> dict[str, Any]:
    n = len(pages)
    if n == 0:
        return {"work_id": candidate["id"], "samples": {}}
    
    kind = candidate["work_kind"]
    samples: dict[str, Any] = {}
    
    def format_sample(idx: int, label: str) -> dict[str, Any]:
        p = pages[idx - 1]
        return {
            "label": label,
            "page_sequence_number": idx,
            "total_pages": n,
            "character_length": sum(block_length(b) for b in p["blocks"]),
            "blocks_count": len(p["blocks"]),
            "preview": text_preview(p, 400),
            "content_hash": p["content_hash"],
        }
    
    if kind == "LONG_FORM":
        samples["start"] = format_sample(1, "Début (Page 1)")
        p25 = max(1, round(n * 0.25))
        samples["25pct"] = format_sample(p25, f"25% (Page {p25})")
        p50 = max(1, round(n * 0.50))
        samples["50pct"] = format_sample(p50, f"50% (Page {p50})")
        p75 = max(1, round(n * 0.75))
        samples["75pct"] = format_sample(p75, f"75% (Page {p75})")
        samples["end"] = format_sample(n, f"Fin (Page {n})")
    else:
        samples["start"] = format_sample(1, "Début (Page 1)")
        if n > 2:
            pmid = max(1, round(n * 0.50))
            samples["middle"] = format_sample(pmid, f"Milieu (Page {pmid})")
        samples["end"] = format_sample(n, f"Fin (Page {n})")
        
    return {
        "work_id": candidate["id"],
        "author": candidate["author"],
        "title": candidate["title"],
        "language": candidate["original_language"],
        "work_kind": kind,
        "total_pages": n,
        "samples": samples,
    }


# =============================================================================
# 7. MAIN ORCHESTRATOR
# =============================================================================
def main() -> None:
    print("=== BOOKFIN CURATED V1 BUILDER ===")
    print(f"Timestamp: {datetime.now(timezone.utc).isoformat()}")
    
    # Prepare directories
    for d in [SOURCES_DIR, NORMALIZED_DIR, PAGES_DIR, SAMPLES_DIR]:
        for lang in ["en", "fr", "es"]:
            (d / lang).mkdir(parents=True, exist_ok=True)
            
    manifest_bytes = CANDIDATES_PATH.read_bytes()
    manifest_sha256 = hashlib.sha256(manifest_bytes).hexdigest()
    manifest = json.loads(manifest_bytes.decode("utf-8"))
    accepted_candidates = [c for c in manifest["candidates"] if c["status"] == "ACCEPTED"]
    accepted_work_ids = set(c["id"] for c in accepted_candidates)

    # -------------------------------------------------------------------------
    # GARDE-FOU STRICT DE DÉMARRAGE : Vérification du verrou de l'édition
    # -------------------------------------------------------------------------
    if LOCK_PATH.exists():
        lock = json.loads(LOCK_PATH.read_text(encoding="utf-8"))
        expected_count = lock["expected_work_count"]
        locked_accepted_ids = set(lock["accepted_work_ids"])
        
        if len(accepted_candidates) != expected_count:
            raise RuntimeError(
                f"REFUS DE DÉMARRAGE : {len(accepted_candidates)} œuvres ACCEPTED trouvées dans {CANDIDATES_PATH}, "
                f"{expected_count} attendues selon le verrou {LOCK_PATH}."
            )
        if accepted_work_ids != locked_accepted_ids:
            missing = locked_accepted_ids - accepted_work_ids
            extra = accepted_work_ids - locked_accepted_ids
            raise RuntimeError(
                f"REFUS DE DÉMARRAGE : Les IDs ACCEPTED ne correspondent pas au verrou {LOCK_PATH}.\n"
                f"  Manquants au manifeste : {missing}\n"
                f"  En trop au manifeste : {extra}"
            )
    else:
        expected_count = len(accepted_candidates)

    if len(accepted_work_ids) != len(accepted_candidates):
        raise RuntimeError("REFUS DE DÉMARRAGE : Des identifiants dupliqués existent parmi les candidats ACCEPTED.")

    # Interdiction stricte de tout candidat non-ACCEPTED
    forbidden_ids = set(c["id"] for c in manifest["candidates"] if c["status"] in {"HOLD", "PROPOSED", "REJECTED"})
    overlap = accepted_work_ids & forbidden_ids
    if overlap:
        raise RuntimeError(f"REFUS DE DÉMARRAGE : Des identifiants HOLD/PROPOSED/REJECTED sont marqués ACCEPTED: {overlap}")
    print(f"Loaded and verified {len(accepted_candidates)} strictly validated ACCEPTED candidates against lockfile.")
    
    acquisition_records: list[dict[str, Any]] = []
    validation_results: list[dict[str, Any]] = []
    control_samples_all: list[dict[str, Any]] = []
    typographical_fixtures: list[dict[str, Any]] = []
    
    total_pages_by_lang = {"en": 0, "fr": 0, "es": 0}
    pages_by_author: dict[str, int] = {}
    pages_by_work: list[dict[str, Any]] = []
    long_pages_count = 0
    short_pages_count = 0
    
    for i, c in enumerate(accepted_candidates, 1):
        cid = c["id"]
        lang = c["original_language"]
        author = c["author"]
        title = c["title"]
        kind = c["work_kind"]
        
        print(f"\n[{i:02d}/{len(accepted_candidates)}] Processing: [{cid}] {author} — {title} ({lang})")
        
        spec = get_download_spec(c)
        source_ext = "html"
        source_file = SOURCES_DIR / lang / f"{cid}.{source_ext}"
        
        # 1. Acquisition
        source_bytes: bytes
        retrieved_at: str
        original_filename: str
        content_type: str
        status: str = "ACQUIRED"
        failure_reason: str | None = None
        
        # Check if POC reuse
        if cid in POC_REUSE and (POC_SOURCES_DIR / POC_REUSE[cid]).exists():
            poc_path = POC_SOURCES_DIR / POC_REUSE[cid]
            source_bytes = poc_path.read_bytes()
            retrieved_at = "2026-09-13T12:00:00Z"
            original_filename = POC_REUSE[cid]
            content_type = "text/html; charset=utf-8"
            source_file.write_bytes(source_bytes)
            print(f"  [Acquired] Reused verified POC source archive ({len(source_bytes):,} bytes)")
        elif spec.get("urls"):
            print(f"  [Downloading multi-part] {len(spec['urls'])} parts from Wikisource")
            combined_parts = []
            try:
                for u in spec["urls"]:
                    req = urllib.request.Request(u, headers=HTTP_HEADERS)
                    with urllib.request.urlopen(req, timeout=25) as resp:
                        combined_parts.append(resp.read().decode("utf-8", errors="replace"))
                source_bytes = ("\n<hr class='bookfin-part-break'/>\n".join(combined_parts)).encode("utf-8")
                content_type = "text/html; charset=utf-8"
                original_filename = f"{cid}_combined.html"
                retrieved_at = datetime.now(timezone.utc).isoformat()
                source_file.write_bytes(source_bytes)
                print(f"  [Acquired] Combined {len(combined_parts)} parts ({len(source_bytes):,} bytes)")
            except Exception as e:
                print(f"  [BLOCKED] Acquisition failed: {e}")
                status = "BLOCKED"
                failure_reason = str(e)
                source_bytes = b""
                retrieved_at = datetime.now(timezone.utc).isoformat()
                original_filename = ""
                content_type = ""
        else:
            download_url = spec["url"]
            print(f"  [Downloading] {download_url}")
            try:
                req = urllib.request.Request(download_url, headers=HTTP_HEADERS)
                with urllib.request.urlopen(req, timeout=30) as resp:
                    source_bytes = resp.read()
                    content_type = resp.headers.get("Content-Type", "text/html; charset=utf-8")
                    original_filename = resp.geturl().split("/")[-1] or f"{cid}.html"
                    retrieved_at = datetime.now(timezone.utc).isoformat()
                    source_file.write_bytes(source_bytes)
                    print(f"  [Acquired] Downloaded {len(source_bytes):,} bytes from {resp.geturl()}")
            except Exception as e:
                print(f"  [BLOCKED] Acquisition failed: {e}")
                status = "BLOCKED"
                failure_reason = str(e)
                source_bytes = b""
                retrieved_at = datetime.now(timezone.utc).isoformat()
                original_filename = ""
                content_type = ""

        source_sha256 = sha256_bytes(source_bytes) if source_bytes else ""

        acq_rec = {
            "id": cid,
            "author": author,
            "title": title,
            "language": lang,
            "form": c["form"],
            "work_kind": kind,
            "status": status,
            "failure_reason": failure_reason,
            "source_provider": spec["provider"],
            "source_url": c["source_url"],
            "download_url": spec["url"],
            "source_file": str(source_file.relative_to(ROOT)),
            "source_sha256": source_sha256,
            "file_size_bytes": len(source_bytes),
            "retrieved_at": retrieved_at,
            "content_type": content_type,
            "format_chosen": spec["format"],
            "available_formats": c.get("available_formats", []),
            "format_rationale": spec["rationale"],
        }
        acquisition_records.append(acq_rec)
        
        if status == "BLOCKED":
            continue
            
        # 2. Normalization
        html_str = source_bytes.decode("utf-8", errors="replace")
        
        # Pride & Prejudice start marker
        if cid == "en-austen-pride-prejudice":
            marker = "<h2>PRIDE &amp; PREJUDICE.</h2>"
            if marker in html_str:
                html_str = html_str[html_str.index(marker):]
                
        # Slicing for autonomous works extracted from master volume
        if spec.get("slice_start") and spec.get("slice_end"):
            p1 = html_str.find(spec["slice_start"], spec.get("slice_start_offset", 0))
            if p1 != -1:
                tag_start = html_str.rfind("<h", max(0, p1 - 100), p1)
                if tag_start != -1:
                    p1 = tag_start
                p2 = html_str.find(spec["slice_end"], p1 + len(spec["slice_start"]))
                if p2 != -1:
                    tag_end = html_str.rfind("<h", max(0, p2 - 100), p2)
                    if tag_end != -1:
                        p2 = tag_end
                    html_str = html_str[p1:p2]
                else:
                    html_str = html_str[p1:]
                    
        document = normalize_html_to_document(html_str, source_format=spec["format"])
        doc_hash = sha256_text(canonical_json(document))
        norm_payload = {
            "work_id": cid,
            "author": author,
            "title": title,
            "language": lang,
            "work_kind": kind,
            "edition_id": spec["provider"].lower().replace(" ", "_"),
            "document_sha256": doc_hash,
            "source_sha256": source_sha256,
            "document": document,
        }
        norm_file = NORMALIZED_DIR / lang / f"{cid}.json"
        norm_file.write_text(json.dumps(norm_payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        print(f"  [Normalized] {len(document['blocks'])} blocks -> {norm_file.relative_to(ROOT)}")
        
        # 3. Pagination
        pages = paginate_document(
            document,
            work_id=cid,
            edition_id=spec["provider"].lower().replace(" ", "_"),
            language=lang,
            target_characters=1600,
        )
        pages_payload = {
            "work_id": cid,
            "author": author,
            "title": title,
            "language": lang,
            "work_kind": kind,
            "edition_id": spec["provider"].lower().replace(" ", "_"),
            "source_sha256": source_sha256,
            "document_sha256": doc_hash,
            "pages_count": len(pages),
            "pages": pages,
        }
        pages_file = PAGES_DIR / lang / f"{cid}_pages.json"
        pages_file.write_text(json.dumps(pages_payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        print(f"  [Paginated] {len(pages)} pages generated -> {pages_file.relative_to(ROOT)}")
        
        # 4. Automated Quality Validation
        val = validate_curated_work(c, document, pages, source_bytes)
        validation_results.append(val)
        if val["issues"]:
            print(f"  [Validation Warnings] {val['issues']}")
        else:
            print(f"  [Validation OK] Valid: {val['valid']}")

        # 5. Human Control Previews
        samples = generate_control_samples(c, pages)
        control_samples_all.append(samples)
        samples_file = SAMPLES_DIR / lang / f"{cid}_samples.json"
        samples_file.write_text(json.dumps(samples, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        
        # 6. Typographical Test Fixture Candidates
        # Find pages with interesting typography
        for p in pages:
            has_it = any(any(s.get("italic") for s in b.get("spans", [])) for b in p["blocks"])
            has_head = any(b.get("type") == "heading" for b in p["blocks"])
            has_break = any(b.get("type") == "scene_break" for b in p["blocks"])
            has_quote = any(b.get("type") == "blockquote" for b in p["blocks"])
            if (has_it and has_head) or has_break or has_quote:
                if len(typographical_fixtures) < 12:
                    typographical_fixtures.append({
                        "work_id": cid,
                        "author": author,
                        "title": title,
                        "language": lang,
                        "page_sequence_number": p["page_sequence_number"],
                        "features": {
                            "has_italic": has_it,
                            "has_heading": has_head,
                            "has_scene_break": has_break,
                            "has_blockquote": has_quote,
                        },
                        "blocks": p["blocks"],
                    })
        
        # Urn counters
        p_count = len(pages)
        total_pages_by_lang[lang] += p_count
        pages_by_author[author] = pages_by_author.get(author, 0) + p_count
        if kind == "LONG_FORM":
            long_pages_count += p_count
        else:
            short_pages_count += p_count
            
        pages_by_work.append({
            "work_id": cid,
            "author": author,
            "title": title,
            "language": lang,
            "work_kind": kind,
            "real_pages_v2": p_count,
            "estimated_pages_midpoint": round((float(c["estimated_bookfin_pages"].split("-")[0]) + float(c["estimated_bookfin_pages"].split("-")[1])) / 2.0, 1),
            "estimated_range": c["estimated_bookfin_pages"],
        })

    # Save Acquisition Report
    acq_report_path = CURATED_DIR / "acquisition_report.json"
    acq_report_data = {
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "total_targets": len(accepted_candidates),
        "acquired_count": sum(1 for r in acquisition_records if r["status"] == "ACQUIRED"),
        "blocked_count": sum(1 for r in acquisition_records if r["status"] == "BLOCKED"),
        "records": acquisition_records,
    }
    acq_report_path.write_text(json.dumps(acq_report_data, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(f"\nWrote acquisition report to {acq_report_path}")

    # Save Mobile Typographical Fixture
    fixture_path = CURATED_DIR / "mobile_preview_fixture.json"
    fixture_data = {
        "description": "Corpus V2 rich typography sample pages for mobile renderer verification",
        "sample_count": len(typographical_fixtures),
        "samples": typographical_fixtures,
    }
    fixture_path.write_text(json.dumps(fixture_data, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(f"Wrote mobile preview fixture to {fixture_path}")

    # Compute Real Urn Statistics
    total_real_pages = sum(total_pages_by_lang.values())
    
    # Difference table
    diff_table = []
    for w in pages_by_work:
        est = w["estimated_pages_midpoint"]
        real = w["real_pages_v2"]
        diff_abs = real - est
        diff_pct = round((diff_abs / est) * 100.0, 1) if est else 0.0
        diff_table.append({
            "work_id": w["work_id"],
            "author": w["author"],
            "title": w["title"],
            "language": w["language"],
            "work_kind": w["work_kind"],
            "estimated_range": w["estimated_range"],
            "estimated_pages_midpoint": est,
            "real_pages_v2": real,
            "diff_absolute": round(diff_abs, 1),
            "diff_percent": diff_pct,
            "urn_share_percent": round((real / total_real_pages) * 100.0, 2) if total_real_pages else 0.0,
        })
    
    # Sort authors by real pages
    authors_ranked = [
        {
            "author": a,
            "real_pages": p,
            "urn_share_percent": round((p / total_real_pages) * 100.0, 2) if total_real_pages else 0.0,
        }
        for a, p in sorted(pages_by_author.items(), key=lambda item: -item[1])
    ]
    
    # Sort works by real pages
    works_ranked = sorted(diff_table, key=lambda w: -w["real_pages_v2"])

    # -------------------------------------------------------------------------
    # GARDE-FOU STRICT DE FIN DE BUILD : ÉCHEC DUR EN CAS DE DIVERGENCE
    # -------------------------------------------------------------------------
    generated_work_ids = set(w["work_id"] for w in diff_table)
    if generated_work_ids != accepted_work_ids:
        missing_ids = accepted_work_ids - generated_work_ids
        extra_ids = generated_work_ids - accepted_work_ids
        raise AssertionError(
            f"ÉCHEC DUR : Divergence entre les œuvres générées et les œuvres ACCEPTED !\n"
            f"  Manquantes ({len(missing_ids)}) : {missing_ids}\n"
            f"  En trop ({len(extra_ids)}) : {extra_ids}"
        )

    # Manifest
    manifest_curated = {
        "manifest_name": "Bookfin Curated Library V1",
        "manifest_version": "1.0",
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "total_works": len(accepted_candidates),
        "total_real_pages": total_real_pages,
        "linguistic_distribution": {
            "en": {
                "pages": total_pages_by_lang["en"],
                "percent": round((total_pages_by_lang["en"] / total_real_pages) * 100.0, 2) if total_real_pages else 0.0,
            },
            "fr": {
                "pages": total_pages_by_lang["fr"],
                "percent": round((total_pages_by_lang["fr"] / total_real_pages) * 100.0, 2) if total_real_pages else 0.0,
            },
            "es": {
                "pages": total_pages_by_lang["es"],
                "percent": round((total_pages_by_lang["es"] / total_real_pages) * 100.0, 2) if total_real_pages else 0.0,
            },
        },
        "work_kind_distribution": {
            "LONG_FORM": {
                "count": 25,
                "pages": long_pages_count,
                "percent": round((long_pages_count / total_real_pages) * 100.0, 2) if total_real_pages else 0.0,
            },
            "SHORT_WORK": {
                "count": 48,
                "pages": short_pages_count,
                "percent": round((short_pages_count / total_real_pages) * 100.0, 2) if total_real_pages else 0.0,
            },
        },
        "top_15_authors": authors_ranked[:15],
        "top_15_works": [
            {
                "work_id": w["work_id"],
                "author": w["author"],
                "title": w["title"],
                "real_pages_v2": w["real_pages_v2"],
                "urn_share_percent": w["urn_share_percent"],
            }
            for w in works_ranked[:15]
        ],
        "works": diff_table,
    }

    manifest_output_path = CURATED_DIR / "manifest.json"
    manifest_output_path.write_text(json.dumps(manifest_curated, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(f"Wrote curated V1 manifest to {manifest_output_path}")
    
    print("\n=== SUMMARY ===")
    print(f"Total Acquired: {acq_report_data['acquired_count']}/{len(accepted_candidates)}")
    print(f"Total Blocked: {acq_report_data['blocked_count']}")
    print(f"Total Real Pages V2: {total_real_pages}")
    print(f"  EN: {total_pages_by_lang['en']} ({manifest_curated['linguistic_distribution']['en']['percent']}%)")
    print(f"  FR: {total_pages_by_lang['fr']} ({manifest_curated['linguistic_distribution']['fr']['percent']}%)")
    print(f"  ES: {total_pages_by_lang['es']} ({manifest_curated['linguistic_distribution']['es']['percent']}%)")


if __name__ == "__main__":
    main()
