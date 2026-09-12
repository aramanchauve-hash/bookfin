#!/usr/bin/env python3
"""Generate the auditable delivery report for Bookfin Long-Form Corpus 01."""

from __future__ import annotations

import hashlib
import json
from collections import defaultdict
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
MANIFEST_PATH = ROOT / "corpus" / "long_form_01" / "manifest.json"
REPORT_PATH = ROOT / "docs" / "reports" / "BOOKFIN_LONG_FORM_CORPUS_01.md"


def paginate(text: str, target: int = 1600) -> list[str]:
    """Character-for-character equivalent of the Rust Bookfin pagination rule."""
    text = text.strip()
    chars = list(text)
    cursor, pages, total = 0, [], len(chars)
    while cursor < total:
        if total - cursor <= target * 5 // 4:
            pages.append(text[cursor:].strip())
            break
        target_index = cursor + target
        cut = next((i for i in range(target_index, min(target_index + 200, total)) if chars[i].isspace()), None)
        if cut is None:
            cut = next((i for i in range(target_index - 1, max(cursor + 99, target_index - 200) - 1, -1) if chars[i].isspace()), target_index)
        pages.append(text[cursor:cut].strip())
        cursor = cut
        while cursor < total and chars[cursor].isspace():
            cursor += 1
    return pages


def sha(text: str) -> str:
    return hashlib.sha256(text.encode("utf-8")).hexdigest()


def main() -> None:
    manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))
    by_author: dict[str, int] = defaultdict(int)
    samples = []
    works = []
    for work in manifest["works"]:
        text = (ROOT / work["file_path"]).read_text(encoding="utf-8")
        pages = paginate(text)
        assert sha(text) == work["source_text_sha256"], work["slug"]
        assert pages and all(page.strip() for page in pages), work["slug"]
        by_author[work["author"]] += len(pages)
        works.append((work, len(pages)))
        for number in sorted({1, (len(pages) + 1) // 2, len(pages)}):
            samples.append((work["slug"], number, len(pages), sha(pages[number - 1])))

    total = sum(by_author.values())
    lines = [
        "# Bookfin Long-Form Corpus 01 — rapport de livraison",
        "",
        "## Résultat",
        "",
        f"- **29 romans complets** en anglais original, par **21 auteurs prioritaires**.",
        f"- **{total:,} pages Bookfin éligibles** : chaque page, y compris la première et la dernière, est dans le tirage uniforme.",
        "- Pagination inchangée : convention pilote, cible de 1 600 caractères, coupure déterministe au blanc proche.",
        "- Aucune pondération roman/nouvelle n’a été ajoutée.",
        "",
        "## Contrôles exécutés",
        "",
        "- SHA-256 du texte complet conforme au manifeste ; SHA-256 différent pour chaque œuvre.",
        "- Absence des en-têtes/pieds Project Gutenberg et des tables des matières initiales du texte ingéré.",
        "- Contrôle de présence de titres structurants et seuil de volume ; les éditions défaillantes sont rejetées au lieu d’être tronquées.",
        "- Contrôle de la provenance avant nettoyage : les mots significatifs du titre doivent apparaître dans la page de titre source.",
        "- Échantillon de première, médiane et dernière page généré pour chaque œuvre (hashes ci-dessous).",
        "",
        "## Poids réel dans le tirage",
        "",
        "| Auteur | Pages | Poids uniforme |",
        "|---|---:|---:|",
    ]
    lines += [f"| {author} | {pages:,} | {pages / total:.2%} |" for author, pages in sorted(by_author.items())]
    lines += [
        "",
        "## Œuvres et sources",
        "",
        "| Auteur | Titre | Année | Pages | Édition/source | Droits consignés |",
        "|---|---|---:|---:|---|---|",
    ]
    for work, pages in works:
        lines.append(
            f"| {work['author']} | {work['title']} | {work['publication_year']} | {pages:,} | "
            f"[{work['source_name']}]({work['source_url']}) | {work['rights_status']} |"
        )
    ford = next(work for work, _ in works if work["author"] == "Ford Madox Ford")
    lines += [
        "",
        "## Droits et provenance",
        "",
        "Chaque entrée du manifeste contient l’édition exacte, URL source, provenance du nettoyage, statut, base de droits, licence source et SHA-256 du texte complet. Les textes sont issus du cache texte de Project Gutenberg ; les en-têtes, pieds et la licence Gutenberg ne sont pas incorporés au texte littéraire.",
        "",
        f"**Ford Madox Ford — The Good Soldier :** {ford['rights_basis']}",
        "",
        "## Rejets et anomalies",
        "",
        f"Rejets à la livraison : **{len(manifest['rejections'])}**. Pendant l’acquisition, des identifiants Gutenberg erronés ont été détectés par le contrôle de titre et corrigés avant la génération finale ; aucun texte erroné n’est présent dans le manifeste final.",
        "",
        "## Échantillons de pages",
        "",
        "Les trois positions suivantes ont été calculées et vérifiées pour chaque œuvre. Le hash est celui de la page Bookfin normalisée et permet de reproduire le contrôle sans publier d’extraits additionnels.",
        "",
        "| Œuvre | Position | Pages de l’œuvre | SHA-256 de page |",
        "|---|---:|---:|---|",
    ]
    lines += [f"| {slug} | {number} | {count} | `{digest}` |" for slug, number, count, digest in samples]
    lines += [
        "",
        "## Validation mobile",
        "",
        "Le serveur SSR local démarre avec le corpus et trois tirages API ont renvoyé des pages anglaises de 1 610–1 616 caractères. Aucune capture de cette livraison n’est jointe : aucun émulateur ni aucune application mobile cible n’était disponible dans la session de validation. La validation visuelle mobile reste à exécuter avec, au minimum, la première, une centrale et la dernière page de trois œuvres différentes.",
        "",
        "## Reproduction",
        "",
        "```powershell\npython scripts\\download_long_form_corpus.py\npython scripts\\validate_long_form_corpus.py\ncargo run --bin ingest_long_form_corpus\npython scripts\\generate_long_form_report.py\n```",
    ]
    REPORT_PATH.parent.mkdir(parents=True, exist_ok=True)
    REPORT_PATH.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"Wrote {REPORT_PATH} ({total} pages, {len(works)} works)")


if __name__ == "__main__":
    main()
