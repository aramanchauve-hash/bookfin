import json
import unittest
from pathlib import Path

from corpus_v2 import document_hash, html_to_document, paginate_document, text_to_document


class CorpusV2Tests(unittest.TestCase):
    def test_html_preserves_heading_italics_and_scene_break(self):
        document = html_to_document("<h2>Chapitre IV</h2><p>Il était <em>très tard</em>.</p><hr><pre>春\n夜</pre>")
        self.assertEqual(document["blocks"][0]["type"], "heading")
        self.assertEqual(document["blocks"][1]["spans"][1], {"text": "très tard", "italic": True})
        self.assertEqual(document["blocks"][2], {"type": "scene_break"})
        self.assertEqual(document["blocks"][3]["type"], "verse")

    def test_pagination_keeps_blocks_and_is_deterministic(self):
        document = text_to_document("Premier paragraphe.\n\nDeuxième paragraphe.")
        self.assertEqual(paginate_document(document, 20), paginate_document(document, 20))
        self.assertEqual(len(paginate_document(document, 20)), 2)

    def test_hash_is_stable_and_unicode_safe(self):
        document = text_to_document("Русский\n\n中文\n\n日本語")
        self.assertEqual(document_hash(document), document_hash(document))

    def test_html_can_keep_only_a_reviewed_content_root(self):
        document = html_to_document(
            "<nav>Navigation</nav><div id='mw-content-text'><h2>Titre</h2><p>Texte.</p></div><footer>Footer</footer>",
            content_root_id="mw-content-text",
        )
        text = "".join(
            span["text"] for block in document["blocks"] for span in block.get("spans", [])
        )
        self.assertIn("TitreTexte.", text)
        self.assertNotIn("Navigation", text)
        self.assertNotIn("Footer", text)

    def test_html_can_exclude_known_boilerplate_and_void_tags(self):
        document = html_to_document(
            "<header id='pg-header'>Gutenberg<img src='x'></header><p>Chapter one.</p><footer id='pg-footer'>License</footer>",
            exclude_ids={"pg-header", "pg-footer"},
        )
        text = "".join(
            span["text"] for block in document["blocks"] for span in block.get("spans", [])
        )
        self.assertEqual(text, "Chapter one.")

    def test_curation_manifest_requires_quality_review_before_epub_preference(self):
        manifest = json.loads(
            (Path(__file__).resolve().parents[1] / "corpus/curation/library_v1_candidates.json").read_text(
                encoding="utf-8"
            )
        )
        policy = manifest["source_acquisition_policy"]
        self.assertEqual(policy["ordered_preference"][0], "EPUB")
        self.assertIn("italics", policy["comparison_required_when_epub_and_html_exist"])
        self.assertIn("TXT", policy["legacy_txt_rule"])
        self.assertIn("BOOKFIN_JSON_NORMALISATION", policy["mandatory_pipeline"])
        required_candidate_fields = {
            "author", "title", "original_language", "form", "publication_year",
            "status", "priority", "preferred_source", "source_provider", "source_url",
            "available_formats", "rights_status", "rights_basis", "estimated_length",
            "estimated_bookfin_pages", "work_kind", "source_hash",
        }
        self.assertGreaterEqual(len(manifest["candidates"]), 80)
        for candidate in manifest["candidates"]:
            self.assertTrue(required_candidate_fields.issubset(candidate))
            self.assertIn(candidate["status"], {"PROPOSED", "HOLD", "ACCEPTED"})
            self.assertIn(candidate["work_kind"], {"LONG_FORM", "SHORT_WORK"})
        self.assertEqual(sum(1 for c in manifest["candidates"] if c["status"] == "ACCEPTED"), 73)
        self.assertEqual(sum(1 for c in manifest["candidates"] if c["status"] == "HOLD"), 7)
        self.assertEqual(sum(1 for c in manifest["candidates"] if c["status"] == "PROPOSED"), 47)
        self.assertEqual(sum(1 for c in manifest["candidates"] if c["status"] == "REJECTED"), 0)

        # ---------------------------------------------------------------------
        # UNICITÉ GLOBALE DES IDS : Indépendamment du statut
        # ---------------------------------------------------------------------
        all_ids = [c["id"] for c in manifest["candidates"]]
        self.assertEqual(len(all_ids), 127, "Le manifeste de curation doit contenir exactement 127 candidats au total")
        self.assertEqual(
            len(all_ids),
            len(set(all_ids)),
            f"Violation: doublons d'identifiants détectés parmi les 127 candidats: {[id_ for id_ in all_ids if all_ids.count(id_) > 1]}"
        )

    def test_curated_v1_corpus_offline_integrity(self):
        root = Path(__file__).resolve().parents[1]
        manifest_path = root / "corpus/curated_v1/manifest.json"
        self.assertTrue(manifest_path.exists())
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
        
        candidates = json.loads(
            (root / "corpus/curation/library_v1_candidates.json").read_text(encoding="utf-8")
        )["candidates"]
        accepted = [c for c in candidates if c["status"] == "ACCEPTED"]
        self.assertEqual(len(accepted), 73)
        self.assertEqual(manifest["total_works"], 73)
        
        total_pages = 0
        all_seen_hashes = set()
        
        for c in accepted:
            wid = c["id"]
            lang = c["original_language"]
            norm_p = root / "corpus" / "curated_v1" / "normalized" / lang / f"{wid}.json"
            pages_p = root / "corpus" / "curated_v1" / "pages" / lang / f"{wid}_pages.json"
            
            self.assertTrue(norm_p.exists(), f"Normalized file missing for {wid}")
            self.assertTrue(pages_p.exists(), f"Pages file missing for {wid}")
            
            norm_doc = json.loads(norm_p.read_text(encoding="utf-8"))
            pages_doc = json.loads(pages_p.read_text(encoding="utf-8"))
            
            blocks = norm_doc["document"]["blocks"]
            self.assertGreater(len(blocks), 0, f"Work {wid} has zero blocks")
            
            pages = pages_doc["pages"]
            self.assertGreater(len(pages), 0, f"Work {wid} has zero pages")
            total_pages += len(pages)
            
            work_hashes = set()
            for idx, p in enumerate(pages, 1):
                self.assertEqual(p["page_sequence_number"], idx, f"Discontinuous sequence in {wid}")
                chash = p.get("content_hash")
                self.assertTrue(bool(chash), f"Missing hash in {wid} page {idx}")
                self.assertNotIn(chash, work_hashes, f"Duplicate page hash inside {wid}: page {idx}")
                work_hashes.add(chash)
                
                # Verify page is not empty
                char_len = sum(len(s.get("text", "")) for b in p["blocks"] for s in b.get("spans", []))
                self.assertGreater(char_len, 0, f"Empty page in {wid} page {idx}")
                
            # Verify no boilerplate in normalized blocks
            full_txt = " ".join("".join(s.get("text", "") for s in b.get("spans", [])) for b in blocks)
            self.assertNotIn("START OF THE PROJECT GUTENBERG", full_txt)
            self.assertNotIn("END OF THE PROJECT GUTENBERG", full_txt)
            self.assertNotIn("Project Gutenberg License", full_txt)
            
        self.assertEqual(total_pages, manifest["total_real_pages"])
        self.assertGreater(total_pages, 10000)
        
        # Verify multilingual fixture exists and covers EN, FR, ES
        fixture_p = root / "corpus/curated_v1/mobile_preview_fixture.json"
        self.assertTrue(fixture_p.exists())
        fixture = json.loads(fixture_p.read_text(encoding="utf-8"))
        langs_covered = {s["language"] for s in fixture["samples"]}
        self.assertEqual(langs_covered, {"en", "fr", "es"})

    def test_curated_v1_strictly_matches_accepted_and_forbids_hold_proposed_rejected(self):
        """Vérifie formellement que curated_v1 contient STRICTEMENT les 73 ACCEPTED et AUCUN HOLD/PROPOSED/REJECTED."""
        root = Path(__file__).resolve().parents[1]
        candidates = json.loads(
            (root / "corpus/curation/library_v1_candidates.json").read_text(encoding="utf-8")
        )["candidates"]

        accepted_ids = {c["id"] for c in candidates if c.get("status") == "ACCEPTED"}
        hold_ids = {c["id"] for c in candidates if c.get("status") == "HOLD"}
        proposed_ids = {c["id"] for c in candidates if c.get("status") == "PROPOSED"}
        rejected_ids = {c["id"] for c in candidates if c.get("status") == "REJECTED"}

        self.assertEqual(len(accepted_ids), 73, "Le nombre d'œuvres ACCEPTED doit être exactement 73")
        self.assertGreater(len(hold_ids), 0, "Il doit y avoir des œuvres en statut HOLD")

        # Vérification du manifeste curated_v1
        manifest = json.loads((root / "corpus/curated_v1/manifest.json").read_text(encoding="utf-8"))
        curated_ids = {w["work_id"] for w in manifest.get("works", [])}

        self.assertEqual(curated_ids, accepted_ids, "curated_ids doit être strictement identique à accepted_ids")
        self.assertEqual(len(accepted_ids - curated_ids), 0, "Aucune œuvre ACCEPTED ne doit manquer")
        self.assertEqual(len(curated_ids - accepted_ids), 0, "Aucune œuvre non-ACCEPTED ne doit être présente")

        # Interdiction formelle de HOLD, PROPOSED et REJECTED dans le manifeste
        self.assertEqual(len(curated_ids & hold_ids), 0, f"Violation: Œuvres en HOLD détectées dans curated_v1: {curated_ids & hold_ids}")
        self.assertEqual(len(curated_ids & proposed_ids), 0, f"Violation: Œuvres en PROPOSED détectées dans curated_v1: {curated_ids & proposed_ids}")
        self.assertEqual(len(curated_ids & rejected_ids), 0, f"Violation: Œuvres en REJECTED détectées dans curated_v1: {curated_ids & rejected_ids}")

        # Vérification des fichiers sur disque
        curated_dir = root / "corpus" / "curated_v1"
        for folder, ext in [("sources", ".html"), ("normalized", ".json"), ("pages", "_pages.json")]:
            for p in (curated_dir / folder).glob(f"*/*{ext}"):
                file_stem = p.name.replace("_pages.json", "").replace(".html", "").replace(".json", "")
                self.assertNotIn(file_stem, hold_ids, f"Fichier HOLD interdit présent sur disque: {p}")
                self.assertNotIn(file_stem, proposed_ids, f"Fichier PROPOSED interdit présent sur disque: {p}")
                self.assertNotIn(file_stem, rejected_ids, f"Fichier REJECTED interdit présent sur disque: {p}")
                self.assertIn(file_stem, accepted_ids, f"Fichier non-ACCEPTED présent sur disque: {p}")

        # Vérification de l'audit d'intégrité
        audit_file = curated_dir / "integrity_audit_report.json"
        if audit_file.exists():
            audit = json.loads(audit_file.read_text(encoding="utf-8"))
            audit_ids = {item["work_id"] for item in audit}
            self.assertEqual(audit_ids, accepted_ids, "audit_ids doit être strictement identique à accepted_ids")
            self.assertEqual(len(audit_ids & hold_ids), 0, "Aucun HOLD ne peut figurer dans l'audit")
            self.assertEqual(len(audit_ids & proposed_ids), 0, "Aucun PROPOSED ne peut figurer dans l'audit")

        # Vérification du verrou curated_v1.lock.json
        lock_file = curated_dir / "curated_v1.lock.json"
        self.assertTrue(lock_file.exists(), "Le fichier de verrou curated_v1.lock.json doit exister")
        lock = json.loads(lock_file.read_text(encoding="utf-8"))
        self.assertEqual(lock["expected_work_count"], len(accepted_ids), "Le nombre d'œuvres verrouillé doit correspondre à accepted_ids")
        self.assertEqual(set(lock["accepted_work_ids"]), accepted_ids, "Les IDs verrouillés doivent être strictement identiques à accepted_ids")



if __name__ == "__main__":
    unittest.main()
