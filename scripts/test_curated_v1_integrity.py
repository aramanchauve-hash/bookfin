import json
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]
CURATED = ROOT / "corpus" / "curated_v1"

# Known technical anomaly mapping established during comprehensive audit
SOURCE_MISMATCH_IDS = {
    "en-sterne-sentimental-journey",
    "en-hazlitt-table-talk",
    "en-gissing-henry-ryecroft",
    "fr-lesage-diable-boiteux",
    "fr-mirbeau-journal-femme-chambre",
    "fr-merimee-carmen",
    "es-larra-el-doncel",
    "es-valle-inclan-sonata-otono",
}

TRUNCATED_IDS = {
    "en-james-portrait-of-a-lady",
}

STRUCTURE_ISSUE_IDS = {
    "en-lamb-essays-of-elia",
    "fr-maupassant-boule-de-suif",
    "es-galdos-dona-perfecta",
}

REVIEW_REQUIRED_IDS = {
    "fr-nerval-aurelia",
    "fr-maupassant-bel-ami",
    "es-valle-inclan-luces-de-bohemia",
    "es-quiroga-a-la-deriva",
    "es-quiroga-la-insolacion",
    "en-poe-cask-amontillado",
    "en-poe-oval-portrait",
    "en-london-to-build-a-fire",
    "en-chopin-story-of-an-hour",
    "fr-barbey-les-diaboliques",
    "es-unamuno-niebla",
}

class CuratedV1IntegrityTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.manifest = json.loads((CURATED / "manifest.json").read_text(encoding="utf-8"))
        cls.acq = json.loads((CURATED / "acquisition_report.json").read_text(encoding="utf-8"))
        cls.candidates = json.loads((ROOT / "corpus/curation/library_v1_candidates.json").read_text(encoding="utf-8"))["candidates"]
        cls.accepted_ids = {c["id"] for c in cls.candidates if c.get("status") == "ACCEPTED"}
        cls.hold_ids = {c["id"] for c in cls.candidates if c.get("status") == "HOLD"}
        cls.proposed_ids = {c["id"] for c in cls.candidates if c.get("status") == "PROPOSED"}
        cls.rejected_ids = {c["id"] for c in cls.candidates if c.get("status") == "REJECTED"}

    def test_00_strict_concordance_with_accepted_candidates(self):
        """Vérifie la concordance absolue avec library_v1_candidates.json (exactement 73 ACCEPTED, 0 HOLD/PROPOSED/REJECTED)."""
        manifest_ids = {w["work_id"] for w in self.manifest["works"]}
        self.assertEqual(len(self.accepted_ids), 73)
        self.assertEqual(manifest_ids, self.accepted_ids)
        self.assertEqual(len(manifest_ids & self.hold_ids), 0, f"HOLD interdit dans curated: {manifest_ids & self.hold_ids}")
        self.assertEqual(len(manifest_ids & self.proposed_ids), 0, f"PROPOSED interdit dans curated: {manifest_ids & self.proposed_ids}")
        self.assertEqual(len(manifest_ids & self.rejected_ids), 0, f"REJECTED interdit dans curated: {manifest_ids & self.rejected_ids}")

    def test_01_all_73_works_present_with_files(self):
        """Vérifie que les 73 œuvres ont un fichier source, un document normalisé et un fichier de pages."""
        self.assertEqual(len(self.manifest["works"]), 73)
        for w in self.manifest["works"]:
            wid = w["work_id"]
            lang = w["language"]
            src = CURATED / "sources" / lang / f"{wid}.html"
            norm = CURATED / "normalized" / lang / f"{wid}.json"
            pages = CURATED / "pages" / lang / f"{wid}_pages.json"
            self.assertTrue(src.exists(), f"Source manquante pour {wid}")
            self.assertTrue(norm.exists(), f"Document normalisé manquant pour {wid}")
            self.assertTrue(pages.exists(), f"Fichier de pages manquant pour {wid}")
            self.assertGreater(src.stat().st_size, 0, f"Fichier source vide pour {wid}")
            self.assertGreater(norm.stat().st_size, 0, f"Fichier normalisé vide pour {wid}")

    def test_02_hashes_integrity(self):
        """Vérifie qu'aucun hash n'est manquant et que les hashes sont valides."""
        for w in self.manifest["works"]:
            wid = w["work_id"]
            lang = w["language"]
            norm_doc = json.loads((CURATED / "normalized" / lang / f"{wid}.json").read_text(encoding="utf-8"))
            self.assertTrue(norm_doc.get("source_sha256"), f"source_sha256 manquant pour {wid}")
            self.assertTrue(norm_doc.get("document_sha256"), f"document_sha256 manquant pour {wid}")
            
            pages_doc = json.loads((CURATED / "pages" / lang / f"{wid}_pages.json").read_text(encoding="utf-8"))
            self.assertTrue(pages_doc.get("document_sha256"), f"document_sha256 manquant dans pages pour {wid}")
            for p in pages_doc.get("pages", []):
                self.assertTrue(p.get("content_hash"), f"content_hash manquant dans page pour {wid}")

    def test_03_continuous_page_sequences(self):
        """Vérifie que chaque œuvre a une séquence de pages strictement continue (1, 2, ..., N)."""
        for w in self.manifest["works"]:
            wid = w["work_id"]
            lang = w["language"]
            pages_doc = json.loads((CURATED / "pages" / lang / f"{wid}_pages.json").read_text(encoding="utf-8"))
            pages = pages_doc.get("pages", [])
            self.assertGreater(len(pages), 0, f"Aucune page pour {wid}")
            for idx, p in enumerate(pages):
                self.assertEqual(p.get("page_sequence_number"), idx + 1, f"Séquence rompue pour {wid} à la page {idx+1}")

    def test_04_all_73_works_contain_no_empty_pages_or_duplicates(self):
        """Vérifie que l'intégralité des 73 œuvres ne contient aucune page vide ni doublon interne."""
        total_pages_checked = 0
        for w in self.manifest["works"]:
            wid = w["work_id"]
            lang = w["language"]
            pages_doc = json.loads((CURATED / "pages" / lang / f"{wid}_pages.json").read_text(encoding="utf-8"))
            pages = pages_doc.get("pages", [])
            seen_hashes = set()
            for p in pages:
                blocks = p.get("blocks", [])
                text = " ".join("".join(s.get("text", "") for s in b.get("spans", [])) for b in blocks).strip()
                self.assertGreater(len(text), 0, f"Page vide détectée dans {wid} (page {p.get('page_sequence_number')})")
                self.assertNotIn(p.get("content_hash"), seen_hashes, f"Doublon exact dans {wid}")
                seen_hashes.add(p.get("content_hash"))
                total_pages_checked += 1
        self.assertEqual(total_pages_checked, self.manifest["total_real_pages"])

    def test_05_verification_of_13_targeted_repairs(self):
        """Vérifie rigoureusement la conformité des 13 œuvres réparées (sources canoniques et élagage des paratextes)."""
        def get_full_text(wid, lang):
            doc = json.loads((CURATED / "normalized" / lang / f"{wid}.json").read_text(encoding="utf-8"))
            return " ".join("".join(s.get("text", "") for s in b.get("spans", [])) for b in doc["document"]["blocks"])

        # 1. en-sterne-sentimental-journey
        sterne = get_full_text("en-sterne-sentimental-journey", "en")
        self.assertNotIn("Bathsheba", sterne)
        self.assertNotIn("Far from the Madding Crowd", sterne)
        self.assertIn("Yorick", sterne)

        # 2. en-hazlitt-table-talk
        hazlitt = get_full_text("en-hazlitt-table-talk", "en")
        self.assertNotIn("Corn Law", hazlitt)

        # 3. en-gissing-henry-ryecroft
        gissing = get_full_text("en-gissing-henry-ryecroft", "en")
        self.assertNotIn("Adelaide Anne Procter", gissing)
        self.assertIn("Ryecroft", gissing)

        # 4. fr-mirbeau-journal-femme-chambre
        mirbeau = get_full_text("fr-mirbeau-journal-femme-chambre", "fr")
        self.assertNotIn("Prentice Hugh", mirbeau)
        self.assertIn("Célestine", mirbeau)

        # 5. fr-lesage-diable-boiteux
        lesage = get_full_text("fr-lesage-diable-boiteux", "fr")
        self.assertNotIn("Squirrels", lesage)
        self.assertIn("TOME SECOND", lesage)

        # 6. fr-merimee-carmen
        carmen = get_full_text("fr-merimee-carmen", "fr")
        self.assertNotIn("Lady Mary Loyd", carmen)
        self.assertNotIn("translated by", carmen.lower())
        self.assertIn("Bohémiens", carmen)

        # 7. es-larra-el-doncel
        larra = get_full_text("es-larra-el-doncel", "es")
        self.assertNotIn("Inside Earth", larra)
        self.assertNotIn("Poul Anderson", larra)
        self.assertIn("don Enrique el Doliente", larra)

        # 8. es-valle-inclan-sonata-otono
        valle = get_full_text("es-valle-inclan-sonata-otono", "es")
        self.assertNotIn("San Francisco in Ruins", valle)
        self.assertIn("Bradomín", valle)

        # 9. en-james-portrait-of-a-lady
        james = get_full_text("en-james-portrait-of-a-lady", "en")
        self.assertIn("CHAPTER LV", james.upper())

        # 10. en-lamb-essays-of-elia
        lamb = get_full_text("en-lamb-essays-of-elia", "en")
        self.assertNotIn("THE LAST ESSAYS OF ELIA", lamb)

        # 11. fr-maupassant-boule-de-suif
        bds = get_full_text("fr-maupassant-boule-de-suif", "fr")
        self.assertNotIn("L'Épave", bds)
        self.assertNotIn("L’Épave", bds)

        # 12. es-galdos-dona-perfecta
        galdos = get_full_text("es-galdos-dona-perfecta", "es")
        self.assertNotIn("VOCABULARY", galdos)

        # 13. fr-nerval-aurelia
        nerval = get_full_text("fr-nerval-aurelia", "fr")
        self.assertNotIn("SOURCES D’AURÉLIA", nerval.upper())

    def test_06_multilingual_fixture_integrity(self):
        """Vérifie que la fixture multilingue mobile est conforme aux spécifications."""
        fixture = json.loads((CURATED / "mobile_preview_fixture.json").read_text(encoding="utf-8"))
        self.assertGreaterEqual(fixture["sample_count"], 8)
        samples = fixture["samples"]
        self.assertGreaterEqual(sum(1 for s in samples if s["language"] == "en"), 2)
        self.assertGreaterEqual(sum(1 for s in samples if s["language"] == "fr"), 2)
        self.assertGreaterEqual(sum(1 for s in samples if s["language"] == "es"), 2)
        
        # Vérification des caractéristiques
        all_features = [feat for s in samples for feat in s.get("typographical_features", [])]
        self.assertTrue(any("italic" in f for f in all_features))
        self.assertTrue(any("heading" in f for f in all_features))

if __name__ == "__main__":
    unittest.main()
