import urllib.request
import re
from pathlib import Path

HEADERS = {'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36'}

def fetch(url: str) -> str:
    req = urllib.request.Request(url, headers=HEADERS)
    with urllib.request.urlopen(req, timeout=25) as resp:
        return resp.read().decode('utf-8', errors='replace')

print("=== 1. BOULE DE SUIF (PG #10746) ===")
bds_html = fetch("https://www.gutenberg.org/cache/epub/10746/pg10746-images.html")
p_bds = bds_html.find("BOULE DE SUIF")
p_next = bds_html.find("AMI PATIENCE", p_bds + 100)
# Find exact heading before AMI PATIENCE
p_h2 = bds_html.rfind("<h2", p_bds, p_next)
slice_bds = bds_html[p_bds:p_h2]
print(f"Boule de Suif start={p_bds}, end={p_h2}, length={len(slice_bds)} bytes")
print("Has incipit:", "Pendant plusieurs jours" in slice_bds)
print("Has excipit (Cornudet):", "Cornudet" in slice_bds[-1000:] and "Marseillaise" in slice_bds[-1000:])
print("No second story:", "Patience" not in slice_bds[-2000:])

print("\n=== 2. ESSAYS OF ELIA (PG #10343) ===")
# Lamb's Essays of Elia (1823) in PG #10343
elia_html = fetch("https://www.gutenberg.org/cache/epub/10343/pg10343-images.html")
print(f"PG #10343 total length: {len(elia_html):,} bytes")
# Check where Essays of Elia starts and ends (and where Last Essays / Notes begin)
p_elia_start = elia_html.find("THE SOUTH-SEA HOUSE")
p_last_essays = elia_html.find("THE LAST ESSAYS OF ELIA")
p_notes = elia_html.find("NOTES</h", p_elia_start)
print(f"South-Sea House (Essay 1) pos: {p_elia_start}")
print(f"The Last Essays of Elia pos: {p_last_essays}")
print(f"Notes start pos: {p_notes}")

print("\n=== 3. DOÑA PERFECTA (PG #15725) ===")
dp_html = fetch("https://www.gutenberg.org/cache/epub/15725/pg15725-images.html")
p_dp_start = dp_html.find("¡VILLAHORRENDA!")
p_notes_dp = dp_html.find("NOTES</h", p_dp_start)
if p_notes_dp == -1:
    p_notes_dp = dp_html.find("id=\"notes\"", p_dp_start)
p_vocab_dp = dp_html.find("VOCABULARY", p_dp_start)
print(f"Doña Perfecta start (Villahorrenda): {p_dp_start}")
print(f"Notes pos: {p_notes_dp}")
print(f"Vocabulary pos: {p_vocab_dp}")
# Check chapter 33 end
p_ch33 = dp_html.find("XXXIII", p_dp_start)
print(f"Chapter 33 pos: {p_ch33}")

print("\n=== 4. AURÉLIA (Wikisource) ===")
aurelia_html = fetch("https://fr.wikisource.org/wiki/Aur%C3%A9lia/Texte_entier")
print(f"Aurélia total length: {len(aurelia_html):,} bytes")
# Check where Part I, Part II are and where Dossier begins
p_part1 = aurelia_html.find("PREMIÈRE PARTIE")
p_part2 = aurelia_html.find("SECONDE PARTIE")
p_dossier = aurelia_html.find("Dossier", p_part2)
p_sources = aurelia_html.find("Les sources d’Aurélia", p_part2)
p_lettres = aurelia_html.find("Lettres à Aurélia", p_part2)
print(f"Part 1 pos: {p_part1}")
print(f"Part 2 pos: {p_part2}")
print(f"Dossier pos: {p_dossier}")
print(f"Sources pos: {p_sources}")
print(f"Lettres pos: {p_lettres}")

