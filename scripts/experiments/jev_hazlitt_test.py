#!/usr/bin/env python3
"""
BOOKFIN — JEV BENCHMARK V2 : TEXT-ONLY VS STRUCTURE-AWARE

Ce script compare le modèle System One Jev (TypeSafe AI) dans deux conditions expérimentales
sur exactement les mêmes cas de normalisation littéraire annotés :
  A. TEXT_ONLY        : texte brut (BEFORE / CANDIDATE / AFTER)
  B. STRUCTURE_AWARE  : tags HTML, line count, preformatted (True/False), attributs DOM objectifs

RÈGLE ABSOLUE : La clé d'annotation humaine `expected` n'est JAMAIS envoyée au modèle.

Pré-requis :
  Variable d'environnement TYPESAFE_API_KEY définie.
  Exemple (PowerShell) :
    $env:TYPESAFE_API_KEY = "votre_cle_ici"
    python scripts/experiments/jev_hazlitt_test.py
"""

import argparse
import json
import os
import sys
import time
import urllib.error
import urllib.request
from typing import Any, Dict, List, Optional, Tuple

# Assurer un affichage console sans crash d'encodage sous Windows
try:
    if sys.stdout.encoding and sys.stdout.encoding.lower() != "utf-8":
        sys.stdout.reconfigure(encoding="utf-8", errors="replace")
    if sys.stderr.encoding and sys.stderr.encoding.lower() != "utf-8":
        sys.stderr.reconfigure(encoding="utf-8", errors="replace")
except Exception:
    pass

DEFAULT_ENDPOINT = "https://api.typesafe.ai/v1/systemone"
DEFAULT_MODEL = "jev-latest"

# Classes de classification autorisées
CRITERIA = {
    "VERSE_QUOTE": "A quoted line or stanza of verse/poetry embedded within the prose.",
    "PROSE_CONTINUATION": "A standard continuation or broken fragment of the surrounding prose sentence.",
    "LAYOUT_ARTIFACT": "A formatting artifact, stray line break, page number, header/footer, or typesetting glitch.",
    "PARATEXT": "An editorial note, footnote, citation, section title, or paratextual label.",
    "UNCERTAIN": "Ambiguous or impossible to classify with high certainty.",
}

# 12 cas réels issus du corpus Curated V1 (Hazlitt, O. Henry, Poe, Sterne, etc.)
TEST_CASES: List[Dict[str, Any]] = [
    {
        "id": "hazlitt-table-talk-01",
        "author": "William Hazlitt",
        "work": "Table-Talk (1821) - Essay 'On the Past and Future'",
        "expected": "VERSE_QUOTE",
        "context_note": "Citation poétique de Dryden ('The Hind and the Panther') en <pre> dans la prose.",
        # TEXT_ONLY
        "before_text": "The ideas we cherish most exist best in a kind of shadowy abstraction,",
        "candidate_text": "Pure in the last recesses of the mind,",
        "after_text": "and derive neither force nor interest from being exposed to public view.\nThey are old familiar acquaintance...",
        # STRUCTURE_AWARE
        "before_dom": "<p>\n  The ideas we cherish most exist best in a kind of shadowy abstraction,\n</p>",
        "candidate_dom": "<pre>\n Pure in the last recesses of the mind,\n</pre>",
        "after_dom": "<p>\n  and derive neither force nor interest from being exposed to public view.\n  They are old familiar acquaintance...\n</p>",
        "dom_info": {
            "tag": "pre",
            "preformatted": True,
            "line_count": 1,
            "classes": [],
            "id": None,
        },
    },
    {
        "id": "hazlitt-table-talk-02",
        "author": "William Hazlitt",
        "work": "Table-Talk (1821) - Essay 'On the Pleasure of Painting'",
        "expected": "VERSE_QUOTE",
        "context_note": "Vers de John Donne cité au sein d'un développement critique sur Andrea del Sarto.",
        # TEXT_ONLY
        "before_text": "where every touch appears conscious of its charge, emulous of truth, and where the painful artist has so distinctly wrought,",
        "candidate_text": "That you might almost say his picture thought.",
        "after_text": "In the one case the colours seem breathed on the canvas as if by magic, the work and the wonder of a moment...",
        # STRUCTURE_AWARE
        "before_dom": "<p>\n  where every touch appears conscious of its charge, emulous of truth, and\n  where the painful artist has so distinctly wrought,\n</p>",
        "candidate_dom": "<pre>\n That you might almost say his picture thought.\n</pre>",
        "after_dom": "<p>\n  In the one case the colours seem breathed on the canvas as if by magic,\n  the work and the wonder of a moment...\n</p>",
        "dom_info": {
            "tag": "pre",
            "preformatted": True,
            "line_count": 1,
            "classes": [],
            "id": None,
        },
    },
    {
        "id": "hazlitt-table-talk-03",
        "author": "William Hazlitt",
        "work": "Table-Talk (1821) - Essay 'On Landscape-Painters'",
        "expected": "VERSE_QUOTE",
        "context_note": "Distique poétique de James Thomson ('The Castle of Indolence') sur deux lignes dans <pre>.",
        # TEXT_ONLY
        "before_text": "The painter not only takes a delight in nature, he has a new and exquisite source of pleasure opened to him in the study and contemplation of works of art—",
        "candidate_text": "Whate'er Lorraine light touch'd with soft'ning hue,\nOr savage Rosa dash'd, or learned Poussin drew.",
        "after_text": "He turns aside to view a country gentleman's seat with eager looks, thinking it may contain some of the rich products of art.",
        # STRUCTURE_AWARE
        "before_dom": "<p>\n  The painter not only takes a delight in nature, he has a new and exquisite\n  source of pleasure opened to him in the study and contemplation of works\n  of art—\n</p>",
        "candidate_dom": "<pre>\n Whate'er Lorraine light touch'd with soft'ning hue,\n Or savage Rosa dash'd, or learned Poussin drew.\n</pre>",
        "after_dom": "<p>\n  He turns aside to view a country gentleman's seat with eager looks,\n  thinking it may contain some of the rich products of art.\n</p>",
        "dom_info": {
            "tag": "pre",
            "preformatted": True,
            "line_count": 2,
            "classes": [],
            "id": None,
        },
    },
    {
        "id": "hazlitt-spenser-anomaly-04",
        "author": "William Hazlitt",
        "work": "Table-Talk (1821) - Essay 'On the Ignorance of the Learned'",
        "expected": "VERSE_QUOTE",
        "context_note": "Vers de Spenser ('Faerie Queene') inséré dans le flux mais tagué anormalement en <p> dans la source Gutenberg.",
        # TEXT_ONLY
        "before_text": "He is impervious to any ray of liberal knowledge; his understanding is",
        "candidate_text": "Not pierceable by power of any star—",
        "after_text": "but it is porous to all sorts of tricks, chicanery, stratagems, and knavery, by which anything is to be got.",
        # STRUCTURE_AWARE
        "before_dom": "<p>\n  He is impervious to any ray of liberal\n  knowledge; his understanding is\n</p>",
        "candidate_dom": "<p>\n  Not pierceable by power of any star—\n</p>",
        "after_dom": "<p>\n  but it is porous to all sorts of tricks, chicanery, stratagems, and\n  knavery, by which anything is to be got.\n</p>",
        "dom_info": {
            "tag": "p",
            "preformatted": False,
            "line_count": 1,
            "classes": [],
            "id": None,
        },
    },
    {
        "id": "hazlitt-prose-continuation-05",
        "author": "William Hazlitt",
        "work": "Table-Talk (1821) - Essay 'On the Ignorance of the Learned'",
        "expected": "PROSE_CONTINUATION",
        "context_note": "Continuation de phrase de prose commençant par une minuscule ('but it is porous...') après une citation intercalée.",
        # TEXT_ONLY
        "before_text": "He is impervious to any ray of liberal knowledge; his understanding is\nNot pierceable by power of any star—",
        "candidate_text": "but it is porous to all sorts of tricks, chicanery, stratagems, and knavery, by which anything is to be got.",
        "after_text": "Mrs. Peachum, indeed, says, that to succeed at the gaming-table, the candidate should have the education of a nobleman.",
        # STRUCTURE_AWARE
        "before_dom": "<p>\n  Not pierceable by power of any star—\n</p>",
        "candidate_dom": "<p>\n  but it is porous to all sorts of tricks, chicanery, stratagems, and\n  knavery, by which anything is to be got.\n</p>",
        "after_dom": "<p>\n  Mrs. Peachum, indeed, says, that\n  to succeed at the gaming-table, the candidate should have the education of\n  a nobleman.\n</p>",
        "dom_info": {
            "tag": "p",
            "preformatted": False,
            "line_count": 2,
            "classes": [],
            "id": None,
        },
    },
    {
        "id": "o-henry-prose-continuation-06",
        "author": "O. Henry",
        "work": "The Gift of the Magi (1906)",
        "expected": "PROSE_CONTINUATION",
        "context_note": "Phrase brisée en cours de route par un saut de page physique (from which ... no mortal finger).",
        # TEXT_ONLY
        "before_text": "In the vestibule below was a letter-box into which no letter would go, and an electric button from which",
        "candidate_text": "no mortal finger could coax a ring.",
        "after_text": "Also appertaining thereunto was a card bearing the name 'Mr. James Dillingham Young.'",
        # STRUCTURE_AWARE
        "before_dom": "<p>In the vestibule below was a letter-box into which no letter would go, and an electric button from which</p>",
        "candidate_dom": "<p>no mortal finger could coax a ring. Also appertaining thereunto was a card bearing the name 'Mr. James Dillingham Young.'</p>",
        "after_dom": "<p>The 'Dillingham' had been flung to the breeze during a former period of prosperity...</p>",
        "dom_info": {
            "tag": "p",
            "preformatted": False,
            "line_count": 1,
            "classes": [],
            "id": None,
        },
    },
    {
        "id": "o-henry-pagenum-07",
        "author": "O. Henry",
        "work": "The Gift of the Magi (1906)",
        "expected": "LAYOUT_ARTIFACT",
        "context_note": "Numéro de page physique '17' intercalé dans le DOM entre deux fragments de phrase.",
        # TEXT_ONLY
        "before_text": "In the vestibule below was a letter-box into which no letter would go, and an electric button from which",
        "candidate_text": "17",
        "after_text": "no mortal finger could coax a ring. Also appertaining thereunto was a card bearing the name 'Mr. James Dillingham Young.'",
        # STRUCTURE_AWARE
        "before_dom": "<p>In the vestibule below was a letter-box into which no letter would go, and an electric button from which</p>",
        "candidate_dom": '<span class="pagenum ws-pagenum" id="17" data-page-number="17">17</span>',
        "after_dom": "<p>no mortal finger could coax a ring. Also appertaining thereunto was a card bearing the name 'Mr. James Dillingham Young.'</p>",
        "dom_info": {
            "tag": "span",
            "preformatted": False,
            "line_count": 1,
            "classes": ["pagenum", "ws-pagenum"],
            "id": "17",
        },
    },
    {
        "id": "poe-cask-pagenum-08",
        "author": "Edgar Allan Poe",
        "work": "The Cask of Amontillado (1846)",
        "expected": "LAYOUT_ARTIFACT",
        "context_note": "Balise de jalon de pagination Gutenberg intercalée entre paragraphes.",
        # TEXT_ONLY
        "before_text": "I was skilful in the Italian vintages myself, and bought largely whenever I could.",
        "candidate_text": "[Page 24]",
        "after_text": "It was about dusk, one evening during the supreme madness of the carnival season, that I encountered my friend.",
        # STRUCTURE_AWARE
        "before_dom": "<p>I was skilful in the Italian vintages myself, and bought largely whenever I could.</p>",
        "candidate_dom": '<span class="pagenum"><a id="Page_24"></a>[Page 24]</span>',
        "after_dom": "<p>It was about dusk, one evening during the supreme madness of the carnival season, that I encountered my friend.</p>",
        "dom_info": {
            "tag": "span",
            "preformatted": False,
            "line_count": 1,
            "classes": ["pagenum"],
            "id": None,
        },
    },
    {
        "id": "hazlitt-essay-heading-09",
        "author": "William Hazlitt",
        "work": "Table-Talk (1821)",
        "expected": "PARATEXT",
        "context_note": "Titre et numérotation d'essai (h2) marquant une division éditoriale.",
        # TEXT_ONLY
        "before_text": "Leonardo, in the slow advances of his, had lived long enough!",
        "candidate_text": "ESSAY II. THE SAME SUBJECT CONTINUED",
        "after_text": "The painter not only takes a delight in nature, he has a new and exquisite source of pleasure opened to him...",
        # STRUCTURE_AWARE
        "before_dom": "<p>Leonardo, in the slow advances of his, had lived long enough!</p>",
        "candidate_dom": "<h2>ESSAY II. THE SAME SUBJECT CONTINUED</h2>",
        "after_dom": "<p>The painter not only takes a delight in nature, he has a new and exquisite source of pleasure opened to him...</p>",
        "dom_info": {
            "tag": "h2",
            "preformatted": False,
            "line_count": 1,
            "classes": [],
            "id": None,
        },
    },
    {
        "id": "sterne-footnote-10",
        "author": "Laurence Sterne",
        "work": "A Sentimental Journey (1768)",
        "expected": "PARATEXT",
        "context_note": "Note de bas de page explicative de l'auteur sur le droit d'aubaine en France.",
        # TEXT_ONLY
        "before_text": "There is nothing more pleasing to a traveller than to find that what he thought an obstacle was only a diversion.",
        "candidate_text": "[557] All the effects of strangers (Swiss and Scotch excepted) dying in France, are seized by virtue of this law, though the heir be upon the spot—the profit of these contingencies being farmed, there is no redress.",
        "after_text": "Now, as the notary had taken care to provide for this in the contract, I went forward with an easy heart.",
        # STRUCTURE_AWARE
        "before_dom": "<p>There is nothing more pleasing to a traveller than to find that what he thought an obstacle was only a diversion.</p>",
        "candidate_dom": '<p class="footnote"><a class="footnote pginternal" href="#citation557">[557]</a> All the effects of strangers (Swiss and Scotch excepted) dying in France, are seized by virtue of this law, though the heir be upon the spot—the profit of these contingencies being farmed, there is no redress.</p>',
        "after_dom": "<p>Now, as the notary had taken care to provide for this in the contract, I went forward with an easy heart.</p>",
        "dom_info": {
            "tag": "p",
            "preformatted": False,
            "line_count": 3,
            "classes": ["footnote"],
            "id": None,
        },
    },
    {
        "id": "gutenberg-transcriber-note-11",
        "author": "Project Gutenberg / Editorial",
        "work": "Table-Talk transcription apparatus",
        "expected": "PARATEXT",
        "context_note": "Notice de transcription Gutenberg encadrée dans une balise transnote.",
        # TEXT_ONLY
        "before_text": "End of the Project Gutenberg EBook of Table-Talk, by William Hazlitt",
        "candidate_text": "[Transcriber's Note: Obvious typographical errors have been corrected. A list of corrections appears at the end of the text.]",
        "after_text": "*** END OF THE PROJECT GUTENBERG EBOOK TABLE-TALK ***",
        # STRUCTURE_AWARE
        "before_dom": "<p>End of the Project Gutenberg EBook of Table-Talk, by William Hazlitt</p>",
        "candidate_dom": '<div class="transnote"><p>[Transcriber\'s Note: Obvious typographical errors have been corrected. A list of corrections appears at the end of the text.]</p></div>',
        "after_dom": "<p>*** END OF THE PROJECT GUTENBERG EBOOK TABLE-TALK ***</p>",
        "dom_info": {
            "tag": "div",
            "preformatted": False,
            "line_count": 2,
            "classes": ["transnote"],
            "id": None,
        },
    },
    {
        "id": "hazlitt-uncertain-motto-12",
        "author": "William Hazlitt",
        "work": "Table-Talk (1821)",
        "expected": "UNCERTAIN",
        "context_note": "Fragment de poème de Sir Edward Dyer cité directement au fil de la prose sans balise de vers ni saut de ligne.",
        # TEXT_ONLY
        "before_text": "However I might say with the poet,",
        "candidate_text": "'My mind to me a kingdom is,'",
        "after_text": "yet I have little ambition 'to set a throne or chair of state in the understandings of other men.'",
        # STRUCTURE_AWARE
        "before_dom": "<p>However I might say with the poet, 'My mind to me a kingdom is,' yet I have little ambition...</p>",
        "candidate_dom": "<span>'My mind to me a kingdom is,'</span>",
        "after_dom": "<span>yet I have little ambition 'to set a throne or chair of state in the understandings of other men.'</span>",
        "dom_info": {
            "tag": "span",
            "preformatted": False,
            "line_count": 1,
            "classes": [],
            "id": None,
        },
    },
]


def build_payload_text_only(case: Dict[str, Any], model: str) -> Dict[str, Any]:
    """Construit le payload TEXT_ONLY (texte brut uniquement, sans aucun tag DOM)."""
    state_content = (
        "CONDITION: TEXT_ONLY\n\n"
        f"BEFORE:\n{case['before_text']}\n\n"
        f"CANDIDATE:\n{case['candidate_text']}\n\n"
        f"AFTER:\n{case['after_text']}"
    )

    return {
        "model": model,
        "state": state_content,
        "questions": {
            "candidate_classification": {
                "type": "choice",
                "instructions": (
                    "Classify the CANDIDATE text snippet in its literary context between BEFORE and AFTER. "
                    "Select the category that best describes CANDIDATE."
                ),
                "criteria": CRITERIA,
            }
        },
    }


def build_payload_structure_aware(case: Dict[str, Any], model: str) -> Dict[str, Any]:
    """Construit le payload STRUCTURE_AWARE (tags DOM, attributs objectifs, line count, preformatted)."""
    dom = case["dom_info"]
    classes_str = ", ".join(dom.get("classes", [])) or "none"
    elem_id = dom.get("id") or "none"

    state_content = (
        "CONDITION: STRUCTURE_AWARE\n\n"
        f"BEFORE (Source Tag: {case.get('before_dom', '')[:60]}...):\n"
        f"{case['before_dom']}\n\n"
        f"CANDIDATE (Tag: <{dom['tag']}>, Preformatted: {dom['preformatted']}, "
        f"LineCount: {dom['line_count']}, Classes: [{classes_str}], ID: {elem_id}):\n"
        f"{case['candidate_dom']}\n\n"
        f"AFTER (Source Tag: {case.get('after_dom', '')[:60]}...):\n"
        f"{case['after_dom']}\n\n"
        "OBJECTIVE_DOM_METADATA:\n"
        f"- tag: <{dom['tag']}>\n"
        f"- preformatted: {dom['preformatted']}\n"
        f"- line_count: {dom['line_count']}\n"
        f"- classes: {classes_str}\n"
        f"- id: {elem_id}"
    )

    return {
        "model": model,
        "state": state_content,
        "questions": {
            "candidate_classification": {
                "type": "choice",
                "instructions": (
                    "Classify the CANDIDATE element in its literary and structural DOM context. "
                    "Use both the text meaning and the objective DOM structure (tags, preformatted status, "
                    "line count, attributes) to determine if CANDIDATE is a quoted verse line, prose continuation, "
                    "layout artifact, paratext, or uncertain."
                ),
                "criteria": CRITERIA,
            }
        },
    }


def call_typesafe_api(
    api_key: str, payload: Dict[str, Any], endpoint: str
) -> Tuple[Optional[Dict[str, Any]], float, Optional[str]]:
    """Envoie la requête POST à l'API TypeSafe AI sans jamais logger la clé.
    Retourne (response_dict, latency_ms, error_message).
    """
    data = json.dumps(payload).encode("utf-8")
    req = urllib.request.Request(
        endpoint,
        data=data,
        headers={
            "Authorization": f"Bearer {api_key}",
            "Content-Type": "application/json",
            "User-Agent": "Bookfin-Jev-Benchmark-V2/1.0",
        },
        method="POST",
    )

    t0 = time.perf_counter()
    try:
        with urllib.request.urlopen(req, timeout=30) as resp:
            body = resp.read().decode("utf-8")
            latency_ms = (time.perf_counter() - t0) * 1000.0
            return json.loads(body), latency_ms, None
    except urllib.error.HTTPError as err:
        latency_ms = (time.perf_counter() - t0) * 1000.0
        err_msg = f"HTTP {err.code} ({err.reason})"
        try:
            body = err.read().decode("utf-8")
            if body:
                err_msg += f" - {body}"
        except Exception:
            pass
        return None, latency_ms, err_msg
    except urllib.error.URLError as err:
        latency_ms = (time.perf_counter() - t0) * 1000.0
        return None, latency_ms, f"Réseau: {err.reason}"
    except Exception as err:
        latency_ms = (time.perf_counter() - t0) * 1000.0
        return None, latency_ms, f"Inattendue: {err}"


def extract_decision(response: Optional[Dict[str, Any]]) -> Dict[str, Any]:
    """Extrait le choix, la confiance et les probabilités de la réponse Jev."""
    if not response:
        return {"label": "ERROR", "confidence": 0.0, "probabilities": {}, "model": "unknown"}

    model = response.get("model", "unknown")
    answers = response.get("answers", {})
    ans = answers.get("candidate_classification", {})

    return {
        "label": ans.get("choice") or "UNKNOWN",
        "confidence": float(ans.get("confidence") or 0.0),
        "probabilities": ans.get("probabilities", {}),
        "model": model,
    }


def format_probabilities(probs: Dict[str, float], chosen: str) -> str:
    """Formate les probabilités pour un affichage lisible."""
    if not probs:
        return "  [probabilités non fournies]"
    sorted_probs = sorted(probs.items(), key=lambda x: x[1], reverse=True)
    lines = []
    for label, p in sorted_probs:
        marker = " (choisi)" if label == chosen else ""
        lines.append(f"    - {label:<20} : {p * 100.0:>5.1f}%{marker}")
    return "\n".join(lines)


def run_benchmark(
    cases: List[Dict[str, Any]],
    api_key: str,
    endpoint: str,
    model: str,
    show_raw: bool = False,
    condition_filter: str = "all",
) -> None:
    """Exécute les deux conditions sur les cas de test et produit les métriques comparatives."""
    print("=" * 80)
    print("BOOKFIN — BENCHMARK JEV : TEXT_ONLY vs STRUCTURE_AWARE")
    print(f"Endpoint : {endpoint}")
    print(f"Modèle   : {model}")
    print(f"Cas test : {len(cases)}")
    print("=" * 80)

    results_text: List[Dict[str, Any]] = []
    results_struct: List[Dict[str, Any]] = []

    for idx, case in enumerate(cases, 1):
        print(f"\n[{idx}/{len(cases)}] TEST CASE : {case['id']}")
        print(f"Auteur  : {case.get('author', 'Unknown')} | Œuvre: {case.get('work', '')}")
        print(f"Attendu : {case['expected']}")
        if case.get("context_note"):
            print(f"Contexte: {case['context_note']}")
        print("-" * 80)

        # Condition A: TEXT_ONLY
        res_text = None
        if condition_filter in ("all", "text_only"):
            payload_text = build_payload_text_only(case, model=model)
            resp_t, lat_t, err_t = call_typesafe_api(api_key, payload_text, endpoint)
            if err_t:
                print(f"  [A. TEXT_ONLY] [ERREUR] {err_t} ({lat_t:.1f} ms)")
                res_text = {"label": "ERROR", "confidence": 0.0, "probabilities": {}, "latency": lat_t, "match": False, "error": err_t}
            else:
                dec_t = extract_decision(resp_t)
                dec_t["latency"] = lat_t
                dec_t["match"] = (dec_t["label"] == case["expected"])
                res_text = dec_t
                if show_raw:
                    print("  [RAW TEXT_ONLY]")
                    print(json.dumps(resp_t, indent=2, ensure_ascii=False))

            match_str_t = "MATCH (OK)" if res_text["match"] else f"MISMATCH (Attendu: {case['expected']})"
            print(f"  [A. TEXT_ONLY]")
            print(f"    Label        : {res_text['label']}")
            print(f"    Confiance    : {res_text['confidence']:.2f}")
            print(f"    Latence      : {res_text['latency']:.1f} ms")
            print(f"    Évaluation   : {match_str_t}")
            print(f"    Probabilités :")
            print(format_probabilities(res_text["probabilities"], res_text["label"]))

        # Condition B: STRUCTURE_AWARE
        res_struct = None
        if condition_filter in ("all", "structure_aware"):
            payload_struct = build_payload_structure_aware(case, model=model)
            resp_s, lat_s, err_s = call_typesafe_api(api_key, payload_struct, endpoint)
            if err_s:
                print(f"  [B. STRUCTURE_AWARE] [ERREUR] {err_s} ({lat_s:.1f} ms)")
                res_struct = {"label": "ERROR", "confidence": 0.0, "probabilities": {}, "latency": lat_s, "match": False, "error": err_s}
            else:
                dec_s = extract_decision(resp_s)
                dec_s["latency"] = lat_s
                dec_s["match"] = (dec_s["label"] == case["expected"])
                res_struct = dec_s
                if show_raw:
                    print("  [RAW STRUCTURE_AWARE]")
                    print(json.dumps(resp_s, indent=2, ensure_ascii=False))

            match_str_s = "MATCH (OK)" if res_struct["match"] else f"MISMATCH (Attendu: {case['expected']})"
            print(f"  [B. STRUCTURE_AWARE]")
            print(f"    Label        : {res_struct['label']}")
            print(f"    Confiance    : {res_struct['confidence']:.2f}")
            print(f"    Latence      : {res_struct['latency']:.1f} ms")
            print(f"    Évaluation   : {match_str_s}")
            print(f"    Probabilités :")
            print(format_probabilities(res_struct["probabilities"], res_struct["label"]))

        # Comparaison directe
        if res_text and res_struct and not res_text.get("error") and not res_struct.get("error"):
            c_delta = res_struct["confidence"] - res_text["confidence"]
            sign = "+" if c_delta >= 0 else ""
            print(f"  -> Bilan cas {case['id']} :")
            print(f"     TEXT_ONLY       = {res_text['label']} (conf: {res_text['confidence']:.2f})")
            print(f"     STRUCTURE_AWARE = {res_struct['label']} (conf: {res_struct['confidence']:.2f})")
            print(f"     Delta Confiance = {sign}{c_delta:.2f}")

        if res_text:
            results_text.append({"case": case, "result": res_text})
        if res_struct:
            results_struct.append({"case": case, "result": res_struct})

    # Synthèse globale et métriques
    print_summary_metrics(results_text, results_struct)


def print_summary_metrics(
    results_text: List[Dict[str, Any]], results_struct: List[Dict[str, Any]]
) -> None:
    """Calcule et affiche les métriques comparatives :
    - Accuracy
    - Confidence moyenne
    - False high-confidence decisions (conf >= 0.80 et mismatch)
    - Gain apporté par les métadonnées DOM
    """
    print("\n" + "=" * 80)
    print("RÉSUMÉ DU BENCHMARK COMPARATIF")
    print("=" * 80)

    def compute_stats(results: List[Dict[str, Any]], name: str) -> Dict[str, Any]:
        valid = [r for r in results if not r["result"].get("error")]
        total = len(valid)
        if total == 0:
            return {
                "name": name,
                "total": 0,
                "correct": 0,
                "accuracy": 0.0,
                "mean_conf": 0.0,
                "false_high_conf": [],
            }

        correct = sum(1 for r in valid if r["result"]["match"])
        mean_conf = sum(r["result"]["confidence"] for r in valid) / total
        false_high_conf = [
            r for r in valid
            if not r["result"]["match"] and r["result"]["confidence"] >= 0.80
        ]

        return {
            "name": name,
            "total": total,
            "correct": correct,
            "accuracy": (correct / total) * 100.0,
            "mean_conf": mean_conf,
            "false_high_conf": false_high_conf,
        }

    stats_text = compute_stats(results_text, "A. TEXT_ONLY")
    stats_struct = compute_stats(results_struct, "B. STRUCTURE_AWARE")

    # Tableau comparatif
    print(f"{'Métrique':<35} | {'A. TEXT_ONLY':<18} | {'B. STRUCTURE_AWARE':<18}")
    print("-" * 80)

    t_acc = f"{stats_text['correct']}/{stats_text['total']} ({stats_text['accuracy']:.1f}%)" if stats_text['total'] else "N/A"
    s_acc = f"{stats_struct['correct']}/{stats_struct['total']} ({stats_struct['accuracy']:.1f}%)" if stats_struct['total'] else "N/A"
    print(f"{'Accuracy':<35} | {t_acc:<18} | {s_acc:<18}")

    t_conf = f"{stats_text['mean_conf']:.2f}" if stats_text['total'] else "N/A"
    s_conf = f"{stats_struct['mean_conf']:.2f}" if stats_struct['total'] else "N/A"
    print(f"{'Confiance moyenne':<35} | {t_conf:<18} | {s_conf:<18}")

    t_fhc = f"{len(stats_text['false_high_conf'])} cas" if stats_text['total'] else "N/A"
    s_fhc = f"{len(stats_struct['false_high_conf'])} cas" if stats_struct['total'] else "N/A"
    print(f"{'Fausses certitudes (conf >= 0.80)':<35} | {t_fhc:<18} | {s_fhc:<18}")

    if stats_text["total"] and stats_struct["total"]:
        acc_gain = stats_struct["accuracy"] - stats_text["accuracy"]
        conf_gain = stats_struct["mean_conf"] - stats_text["mean_conf"]
        fhc_reduction = len(stats_text["false_high_conf"]) - len(stats_struct["false_high_conf"])
        print("-" * 80)
        print("GAIN APPORTÉ PAR LES MÉTADONNÉES DOM :")
        print(f"  - Gain Accuracy                   : {acc_gain:+.1f}%")
        print(f"  - Gain Confiance moyenne          : {conf_gain:+.2f}")
        print(f"  - Réduction fausses certitudes    : {fhc_reduction:+d} cas")

        if stats_text["false_high_conf"]:
            print("\nDétail des fausses certitudes (TEXT_ONLY) :")
            for r in stats_text["false_high_conf"]:
                print(f"  - Cas {r['case']['id']} : Prédit '{r['result']['label']}' avec conf={r['result']['confidence']:.2f} au lieu de '{r['case']['expected']}'")

        if stats_struct["false_high_conf"]:
            print("\nDétail des fausses certitudes (STRUCTURE_AWARE) :")
            for r in stats_struct["false_high_conf"]:
                print(f"  - Cas {r['case']['id']} : Prédit '{r['result']['label']}' avec conf={r['result']['confidence']:.2f} au lieu de '{r['case']['expected']}'")

    print("=" * 80)


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Bookfin Benchmark Jev V2 : TEXT_ONLY vs STRUCTURE_AWARE"
    )
    parser.add_argument(
        "--raw",
        action="store_true",
        help="Afficher les réponses JSON brutes de l'API.",
    )
    parser.add_argument(
        "--condition",
        choices=["all", "text_only", "structure_aware"],
        default="all",
        help="Condition expérimentale à évaluer (défaut: all).",
    )
    parser.add_argument(
        "--model",
        type=str,
        default=DEFAULT_MODEL,
        help=f"Modèle TypeSafe Jev (défaut: {DEFAULT_MODEL}).",
    )
    parser.add_argument(
        "--endpoint",
        type=str,
        default=DEFAULT_ENDPOINT,
        help=f"Endpoint TypeSafe AI (défaut: {DEFAULT_ENDPOINT}).",
    )
    parser.add_argument(
        "--case-id",
        type=str,
        default=None,
        help="Exécuter un cas de test spécifique (ex: hazlitt-table-talk-01).",
    )
    args = parser.parse_args()

    api_key = os.environ.get("TYPESAFE_API_KEY")
    if not api_key or not api_key.strip():
        print(
            "ERREUR: La variable d'environnement TYPESAFE_API_KEY est introuvable ou vide.\n"
            "Veuillez définir votre clé TypeSafe AI dans votre environnement avant d'exécuter ce script.\n\n"
            "Exemples :\n"
            "  Windows PowerShell : $env:TYPESAFE_API_KEY = \"votre_cle_ici\"\n"
            "  Linux / macOS      : export TYPESAFE_API_KEY=\"votre_cle_ici\"",
            file=sys.stderr,
        )
        sys.exit(1)

    cases = TEST_CASES
    if args.case_id:
        cases = [c for c in TEST_CASES if c["id"] == args.case_id]
        if not cases:
            print(f"ERREUR: Aucun cas de test trouvé avec l'ID '{args.case_id}'.", file=sys.stderr)
            sys.exit(1)

    run_benchmark(
        cases=cases,
        api_key=api_key,
        endpoint=args.endpoint,
        model=args.model,
        show_raw=args.raw,
        condition_filter=args.condition,
    )


if __name__ == "__main__":
    main()
