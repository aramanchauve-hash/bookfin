"""Report the page-uniform editorial composition of ACCEPTED candidates and simulations.

Bookfin draws pages uniformly: composition IS the algorithm; no hidden algorithmic
weighting is introduced. Each accepted candidate contributes its estimated Bookfin pages.
"""

from __future__ import annotations

import json
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
MANIFEST_PATH = ROOT / "corpus" / "curation" / "library_v1_candidates.json"
OUTPUT_PATH = ROOT / "corpus" / "curation" / "accepted_weight_simulation.json"


def midpoint(page_range: str) -> float:
    low, high = (int(part) for part in page_range.split("-", maxsplit=1))
    return (low + high) / 2


def calculate_scenario(name: str, description: str, candidates: list[dict[str, object]]) -> dict[str, object]:
    total_pages = sum(midpoint(c["estimated_bookfin_pages"]) for c in candidates)

    dim_work = defaultdict(float)
    dim_author = defaultdict(float)
    dim_lang = defaultdict(float)
    dim_kind = defaultdict(float)

    for c in candidates:
        pages = midpoint(c["estimated_bookfin_pages"])
        dim_work[f"{c['title']} ({c['author']})"] += pages
        dim_author[c["author"]] += pages
        dim_lang[c["original_language"]] += pages
        dim_kind[c["work_kind"]] += pages

    def format_distribution(mapping: dict[str, float]) -> list[dict[str, object]]:
        return [
            {
                "key": key,
                "estimated_pages": round(value, 1),
                "share_percent": round(value / total_pages * 100, 2) if total_pages else 0.0,
            }
            for key, value in sorted(mapping.items(), key=lambda item: (-item[1], item[0]))
        ]

    dist_author = format_distribution(dim_author)
    dist_lang = format_distribution(dim_lang)
    dist_kind = format_distribution(dim_kind)
    dist_work = format_distribution(dim_work)

    dominations = [row for row in dist_author if row["share_percent"] >= 15.0]

    return {
        "scenario_name": name,
        "description": description,
        "candidate_count": len(candidates),
        "total_estimated_pages": round(total_pages, 1),
        "long_form_count": sum(1 for c in candidates if c["work_kind"] == "LONG_FORM"),
        "short_work_count": sum(1 for c in candidates if c["work_kind"] == "SHORT_WORK"),
        "languages": {row["key"]: row["share_percent"] for row in dist_lang},
        "work_kinds": {row["key"]: row["share_percent"] for row in dist_kind},
        "potential_author_dominations_gte_15pct": dominations,
        "top_5_authors": dist_author[:5],
        "top_5_works": dist_work[:5],
        "top_10_authors": dist_author[:10],
        "top_10_works": dist_work[:10],
        "all_authors": dist_author,
        "all_works": dist_work,
    }


def main() -> None:
    manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))
    candidates = manifest["candidates"]

    # 1. Live ACCEPTED set
    live_accepted = [c for c in candidates if c["status"] == "ACCEPTED"]

    # 2. Scenario A: Tri-lingual minimal POC pilot (6 works)
    scen_a_ids = {
        "en-austen-pride-prejudice",
        "fr-maupassant-la-parure",
        "fr-maupassant-la-ficelle",
        "es-quiroga-a-la-deriva",
        "es-quiroga-el-almohadon-de-plumas",
        "es-cervantes-rinconete-y-cortadillo",
    }
    scen_a_cands = [c for c in candidates if c["id"] in scen_a_ids]

    # 3. Scenario B: Core P0 requested works
    scen_b_cands = [c for c in candidates if c["priority"] == "P0"]

    # 4. Scenario C: Balanced target library (~25 long, ~48 short works across FR, EN, ES)
    target_long = [
        # EN (8)
        "en-austen-pride-prejudice", "en-austen-persuasion", "en-sterne-sentimental-journey",
        "en-gissing-henry-ryecroft", "en-james-portrait-of-a-lady", "en-hazlitt-table-talk",
        "en-wilde-dorian-gray", "en-lamb-essays-of-elia",
        # FR (9)
        "fr-mirbeau-journal-femme-chambre", "fr-maupassant-bel-ami", "fr-maupassant-pierre-et-jean",
        "fr-schwob-vies-imaginaires", "fr-lesage-diable-boiteux", "fr-diderot-jacques-le-fataliste",
        "fr-huysmans-a-rebours", "fr-stendhal-chartreuse-de-parme", "fr-barbey-les-diaboliques",
        # ES (8)
        "es-cervantes-quijote", "es-quevedo-los-suenos", "es-quevedo-el-buscon",
        "es-larra-el-doncel", "es-pardo-bazan-los-pazos-de-ulloa", "es-galdos-misericordia",
        "es-galdos-dona-perfecta", "es-unamuno-niebla",
    ]
    target_short = [
        # EN (17)
        "en-johnson-rasselas", "en-browne-religio-medici", "en-browne-hydriotaphia",
        "en-james-turn-of-the-screw", "en-james-daisy-miller", "en-james-beast-in-the-jungle",
        "en-james-aspern-papers", "en-poe-tell-tale-heart", "en-poe-cask-amontillado",
        "en-poe-oval-portrait", "en-doyle-scandal-bohemia", "en-doyle-red-headed-league",
        "en-london-to-build-a-fire", "en-henry-gift-of-the-magi", "en-henry-last-leaf",
        "en-saki-open-window", "en-chopin-story-of-an-hour", "en-melville-bartleby",
        # FR (17)
        "fr-maupassant-la-parure", "fr-maupassant-la-ficelle", "fr-maupassant-deux-amis",
        "fr-maupassant-le-horla", "fr-maupassant-boule-de-suif", "fr-maupassant-la-maison-tellier",
        "fr-schwob-livre-de-monelle", "fr-schwob-croisade-des-enfants", "fr-merimee-mateo-falcone",
        "fr-merimee-carmen", "fr-daudet-chevre-seguin", "fr-daudet-secret-cornille",
        "fr-daudet-trois-messes-basses", "fr-diderot-neveu-de-rameau", "fr-nerval-aurelia",
        "fr-bloy-histoires-desobligeantes", "fr-stendhal-chroniques-italiennes",
        # ES (14)
        "es-cervantes-rinconete-y-cortadillo", "es-cervantes-la-gitanilla", "es-cervantes-el-licenciado-vidriera",
        "es-cervantes-el-celoso-extremeno", "es-cervantes-el-coloquio-de-los-perros",
        "es-quiroga-a-la-deriva", "es-quiroga-el-almohadon-de-plumas", "es-quiroga-la-insolacion",
        "es-clarin-adios-cordera", "es-valle-inclan-sonata-estio", "es-unamuno-san-manuel-bueno-martir",
        "es-valle-inclan-sonata-otono", "es-valle-inclan-sonata-primavera", "es-valle-inclan-luces-de-bohemia",
    ]
    scen_c_ids = set(target_long + target_short)
    scen_c_cands = [c for c in candidates if c["id"] in scen_c_ids]

    # 5. Scenario D: Full catalog exploration (all candidates)
    scen_d_cands = list(candidates)

    report: dict[str, object] = {
        "rule": "Bookfin draws pages uniformly: composition IS the algorithm. No algorithmic weighting is applied.",
        "status_gate": f"Statut en direct : {len(live_accepted)} candidats ACCEPTED validés par décision éditoriale humaine (0 en base SQL / zéro import exécuté).",
        "live_accepted_count": len(live_accepted),
        "live_accepted": calculate_scenario(
            "live_status",
            f"Statut actuel en temps réel ({len(live_accepted)} œuvres ACCEPTED validées par curation humaine)",
            live_accepted,
        ),
        "scenarios": [
            calculate_scenario("scenario_a_pilot", "Scénario A : Pilote minimal 3 langues (6 œuvres)", scen_a_cands),
            calculate_scenario("scenario_b_p0_core", "Scénario B : Noyau des œuvres P0 prioritaires (42 œuvres)", scen_b_cands),
            calculate_scenario("scenario_c_linguistic_balance_illustration", "Scénario de composition linguistique équilibrée — illustration uniquement (25 longues, 49 courtes)", scen_c_cands),
            calculate_scenario("scenario_d_full_catalog", "Scénario D : Tous les candidats explorés (127 œuvres, montrant les dominations sans curation)", scen_d_cands),
        ],
    }

    OUTPUT_PATH.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(f"Wrote multi-scenario simulation to {OUTPUT_PATH}")


if __name__ == "__main__":
    main()
