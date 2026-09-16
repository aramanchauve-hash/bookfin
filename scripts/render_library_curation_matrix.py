"""Render the human decision matrix from the curation manifest.

This is a read-only transformation of editorial metadata. It never downloads,
accepts, imports or modifies a candidate status.
"""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
MANIFEST_PATH = ROOT / "corpus" / "curation" / "library_v1_candidates.json"
OUTPUT_PATH = ROOT / "docs" / "reports" / "BOOKFIN_LIBRARY_CURATION_V1_MATRIX.md"


def cell(value: object) -> str:
    return str(value or "—").replace("|", "\\|").replace("\n", " ")


def main() -> None:
    manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))
    rows = [
        "# BOOKFIN — Matrice de décision de curation V1",
        "",
        "Cette annexe est générée directement depuis `corpus/curation/library_v1_candidates.json`.",
        "Tous les statuts restent modifiables dans le manifeste : `PROPOSED`, `ACCEPTED`, `REJECTED`, `HOLD`.",
        "Règle stricte : aucune œuvre ne passe à `ACCEPTED` sans décision humaine explicite.",
        "",
        "| Auteur | Œuvre | Langue | Date | Forme / Classe | Longueur estimée | Pages Bookfin | Source structurée | Formats disponibles | Format recommandé | Droits | Poids réel futur | Statut | Notes |",
        "|---|---|---|---|---|---|---:|---|---|---|---|---|---|---|",
    ]
    for candidate in manifest["candidates"]:
        source_url = candidate.get("source_url")
        provider = candidate.get("source_provider") or "À vérifier"
        if source_url:
            source_link = f"[{provider}]({source_url})"
        else:
            source_link = provider

        formats = ", ".join(candidate.get("available_formats", [])) or "à vérifier"
        notes = candidate.get("notes") or candidate.get("note") or "—"

        rows.append(
            "| "
            + " | ".join(
                [
                    cell(candidate["author"]),
                    cell(candidate["title"]),
                    cell(candidate["original_language"]),
                    cell(candidate["publication_year"]),
                    cell(f"{candidate['form']} · {candidate['work_kind']}"),
                    cell(candidate["estimated_length"]),
                    cell(candidate["estimated_bookfin_pages"]),
                    cell(source_link),
                    cell(formats),
                    cell(candidate["preferred_source"]),
                    cell(candidate["rights_status"]),
                    "Tirage uniforme",
                    cell(candidate["status"]),
                    cell(notes),
                ]
            )
            + " |"
        )

    rows.extend(
        [
            "",
            "## Lecture des poids et principe du tirage uniforme",
            "",
            "Bookfin tire les pages uniformément dans l'urne : **le nombre de pages est le poids éditorial réel**.",
            "Aucune pondération algorithmique cachée ne compense un déséquilibre de composition.",
            "Consultez les simulations multi-scénarios dans `corpus/curation/accepted_weight_simulation.json` et le rapport principal [BOOKFIN_LIBRARY_CURATION_V1.md](BOOKFIN_LIBRARY_CURATION_V1.md).",
            "",
        ]
    )
    OUTPUT_PATH.write_text("\n".join(rows) + "\n", encoding="utf-8")
    print(f"Rendered {len(manifest['candidates'])} rows into {OUTPUT_PATH}")


if __name__ == "__main__":
    main()
