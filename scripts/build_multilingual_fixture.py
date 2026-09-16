import json
from pathlib import Path

curated = Path("corpus/curated_v1/pages")

def get_page(lang, wid, page_num):
    data = json.loads((curated / lang / f"{wid}_pages.json").read_text(encoding="utf-8"))
    for p in data["pages"]:
        if p["page_sequence_number"] == page_num:
            # Extract features
            blocks = p.get("blocks", [])
            has_italic = any(any(s.get("italic") or s.get("em") for s in b.get("spans", [])) for b in blocks)
            has_heading = any(b.get("type") == "heading" for b in blocks)
            has_scene_break = any(b.get("type") == "scene_break" for b in blocks)
            has_blockquote = any(b.get("type") == "blockquote" for b in blocks)
            has_dialogue = any(any(c in "".join(s.get("text", "") for s in b.get("spans", [])) for c in "«»—\"") for b in blocks)
            has_spanish_punct = any(any(c in "".join(s.get("text", "") for s in b.get("spans", [])) for c in "¿¡ñ") for b in blocks)
            has_french_accents = any(any(c in "".join(s.get("text", "") for s in b.get("spans", [])) for c in "éèêàùçœæ") for b in blocks)
            
            return {
                "work_id": wid,
                "author": data["author"],
                "title": data["title"],
                "language": lang,
                "page_sequence_number": page_num,
                "features": {
                    "has_italic": has_italic,
                    "has_heading": has_heading,
                    "has_scene_break": has_scene_break,
                    "has_blockquote": has_blockquote,
                    "has_dialogue": has_dialogue,
                    "has_spanish_punct": has_spanish_punct,
                    "has_french_accents": has_french_accents,
                },
                "blocks": blocks
            }
    raise ValueError(f"Page {page_num} not found in {wid}")

samples = [
    # --- ANGLAIS (4 pages) ---
    # 1. Pride & Prejudice p.1 (Heading, Scene Break, Opening Prose)
    get_page("en", "en-austen-pride-prejudice", 1),
    # 2. Persuasion p.11 (Heading, Italic, Dialogue)
    get_page("en", "en-austen-persuasion", 11),
    # 3. Dorian Gray p.1 (Rich description, italic, dialogue)
    get_page("en", "en-wilde-dorian-gray", 1),
    # 4. Sherlock Holmes - Scandal in Bohemia p.1 (Heading, Italic, Narrative prose)
    get_page("en", "en-doyle-scandal-bohemia", 1),

    # --- FRANÇAIS (4 pages) ---
    # 5. La Chartreuse de Parme p.1 (Heading, French Accents, Narrative prose, Scene break)
    get_page("fr", "fr-stendhal-chartreuse-de-parme", 1),
    # 6. Bel-Ami p.2 (French dialogue with tirets, accents, dialogue flow)
    get_page("fr", "fr-maupassant-bel-ami", 2),
    # 7. À rebours p.1 (Heading, Rich aesthetic vocabulary, italics, accents)
    get_page("fr", "fr-huysmans-a-rebours", 1),
    # 8. Histoires désobligeantes p.4 (French dialogue, guillemets, italics, accents)
    get_page("fr", "fr-bloy-histoires-desobligeantes", 4),

    # --- ESPAGNOL (4 pages) ---
    # 9. Don Quijote de la Mancha p.1 (Classic Spanish opening, accents, archaic spelling)
    get_page("es", "es-cervantes-quijote", 1),
    # 10. Misericordia p.4 (Spanish dialogue with ¿?, ¡!, accents, ñ)
    get_page("es", "es-galdos-misericordia", 4),
    # 11. Los pazos de Ulloa p.2 (Heading, Spanish description, accents, ñ, italics)
    get_page("es", "es-pardo-bazan-los-pazos-de-ulloa", 2),
    # 12. El celoso extremeño p.10 (Dialogue with ¿?, ¡!, ñ, classic Spanish prose)
    get_page("es", "es-cervantes-el-celoso-extremeno", 10),
]

fixture = {
    "description": "Corpus V2 rich multilingual typography validation fixture (English, French, Spanish)",
    "sample_count": len(samples),
    "samples": samples
}

out_path = Path("corpus/curated_v1/mobile_preview_fixture.json")
out_path.write_text(json.dumps(fixture, indent=2, ensure_ascii=False), encoding="utf-8")
print(f"Successfully generated multilingual fixture with {len(samples)} samples at {out_path}")
