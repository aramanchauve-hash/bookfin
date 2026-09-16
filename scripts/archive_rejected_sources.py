import hashlib
import json
import shutil
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CURATED = ROOT / "corpus" / "curated_v1"
REJECTED_DIR = CURATED / "rejected_sources"
REJECTED_DIR.mkdir(parents=True, exist_ok=True)

manifest = json.loads((CURATED / "manifest.json").read_text(encoding="utf-8"))
work_dict = {w["work_id"]: w for w in manifest["works"]}

acq_data = json.loads((CURATED / "acquisition_report.json").read_text(encoding="utf-8"))
acq_dict = {r["id"]: r for r in acq_data.get("records", [])}

BLOCK_METADATA = {
    "en-sterne-sentimental-journey": {
        "reason": "SOURCE_MISMATCH: Téléchargement de PG #107 (Thomas Hardy Far from the Madding Crowd) au lieu de Laurence Sterne.",
        "correction_type": "SOURCE_REPLACEMENT",
    },
    "en-hazlitt-table-talk": {
        "reason": "SOURCE_MISMATCH: Téléchargement de PG #4335 (T.R. Malthus Corn Laws) au lieu de William Hazlitt.",
        "correction_type": "SOURCE_REPLACEMENT",
    },
    "en-gissing-henry-ryecroft": {
        "reason": "SOURCE_MISMATCH: Téléchargement de PG #2304 (Adelaide Anne Procter Legends and Lyrics) au lieu de George Gissing.",
        "correction_type": "SOURCE_REPLACEMENT",
    },
    "fr-mirbeau-journal-femme-chambre": {
        "reason": "SOURCE_MISMATCH: Téléchargement de PG #43170 (F.M. Peard Prentice Hugh en anglais) au lieu d Octave Mirbeau.",
        "correction_type": "SOURCE_REPLACEMENT",
    },
    "fr-lesage-diable-boiteux": {
        "reason": "SOURCE_MISMATCH: Téléchargement de PG #33434 (George Waring The Squirrels en anglais) au lieu d Alain-René Lesage.",
        "correction_type": "SOURCE_REPLACEMENT",
    },
    "fr-merimee-carmen": {
        "reason": "SOURCE_MISMATCH: Téléchargement de PG #2465 (traduction anglaise de Lady Mary Loyd) au lieu du texte original français.",
        "correction_type": "SOURCE_REPLACEMENT",
    },
    "es-larra-el-doncel": {
        "reason": "SOURCE_MISMATCH: Téléchargement de PG #51184 (Poul Anderson Inside Earth en anglais) au lieu de Mariano José de Larra.",
        "correction_type": "SOURCE_REPLACEMENT",
    },
    "es-valle-inclan-sonata-otono": {
        "reason": "SOURCE_MISMATCH: Téléchargement de PG #37537 (San Francisco in Ruins, album photo en anglais) au lieu de Valle-Inclán.",
        "correction_type": "SOURCE_REPLACEMENT",
    },
    "en-james-portrait-of-a-lady": {
        "reason": "TRUNCATED: Volume I (of II) seul téléchargé (PG #2833, chapitres 1 à 27). Volume II manquant (PG #2834, chapitres 28 à 55).",
        "correction_type": "MULTI_VOLUME_ASSEMBLY",
    },
    "en-lamb-essays-of-elia": {
        "reason": "STRUCTURE_ISSUE: Présence de 400 pages de notes critiques universitaires et index de concordance jointes aux essais.",
        "correction_type": "TEXT_EXTRACTION_AND_CLEANING",
    },
    "fr-maupassant-boule-de-suif": {
        "reason": "STRUCTURE_ISSUE: Volume complet Ollendorff de 12 nouvelles téléchargé au lieu de la seule nouvelle éponyme Boule de suif.",
        "correction_type": "TEXT_EXTRACTION_AND_CLEANING",
    },
    "es-galdos-dona-perfecta": {
        "reason": "STRUCTURE_ISSUE: Édition pédagogique américaine contenant des notes d étude et un imposant dictionnaire bilingue espagnol-anglais.",
        "correction_type": "TEXT_EXTRACTION_AND_CLEANING",
    },
    "fr-nerval-aurelia": {
        "reason": "STRUCTURE_ISSUE: Dossier critique posthume joint (Les sources d Aurélia, Lettres à Aurélia) représentant 80% de volume additionnel.",
        "correction_type": "TEXT_EXTRACTION_AND_CLEANING",
    },
}

records = []
for wid, meta in BLOCK_METADATA.items():
    w = work_dict[wid]
    lang = w["language"]
    src_file = CURATED / "sources" / lang / f"{wid}.html"
    
    src_bytes = src_file.read_bytes()
    src_hash = hashlib.sha256(src_bytes).hexdigest()
    
    archive_dest = REJECTED_DIR / f"{wid}_rejected.html"
    shutil.copy2(src_file, archive_dest)
    
    acq_rec = acq_dict.get(wid, {})
    records.append({
        "work_id": wid,
        "author": w["author"],
        "title": w["title"],
        "language": lang,
        "rejected_source_file": str(archive_dest.relative_to(ROOT)),
        "rejected_source_url": acq_rec.get("download_url", acq_rec.get("source_url", "")),
        "rejected_source_sha256": src_hash,
        "rejected_file_bytes": len(src_bytes),
        "rejection_reason": meta["reason"],
        "correction_type": meta["correction_type"],
        "pages_v2_before": w["real_pages_v2"],
        "archived_at": datetime.now(timezone.utc).isoformat(),
    })

ledger = {
    "archive_version": "1.0",
    "archived_at": datetime.now(timezone.utc).isoformat(),
    "rejected_works_count": len(records),
    "records": records,
}

ledger_path = REJECTED_DIR / "rejected_sources_ledger.json"
ledger_path.write_text(json.dumps(ledger, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
print(f"Archived {len(records)} faulty source files to {REJECTED_DIR}")
print(f"Ledger written to {ledger_path}")

