import json
import urllib.request
import urllib.error
from pathlib import Path

manifest = json.loads(Path("corpus/curation/library_v1_candidates.json").read_text(encoding="utf-8"))
accepted = [c for c in manifest["candidates"] if c["status"] == "ACCEPTED"]

headers = {
    "User-Agent": "BookfinCuratedV1/1.0 (https://bookfin.app; contact@bookfin.app) Python/3.12"
}

results = []

for i, c in enumerate(accepted, 1):
    cid = c["id"]
    title = c["title"]
    url = c["source_url"]
    provider = c["source_provider"]
    lang = c["original_language"]
    
    download_url = url
    format_type = "html"
    
    # Check if Gutenberg URL
    if "gutenberg.org/ebooks/" in url:
        ebook_id = url.rstrip("/").split("/")[-1]
        # For Gutenberg: use HTML master view
        download_url = f"https://www.gutenberg.org/ebooks/{ebook_id}.html.images"
        format_type = "gutenberg_html"
    elif "wikisource.org" in url:
        format_type = "wikisource_html"
    
    # Special case: Conan Doyle on Gutenberg has full collection; check if we use Wikisource for the individual story
    if cid == "en-doyle-scandal-bohemia":
        download_url = "https://en.wikisource.org/wiki/The_Adventures_of_Sherlock_Holmes/A_Scandal_in_Bohemia"
        format_type = "wikisource_html"
    elif cid == "en-doyle-red-headed-league":
        download_url = "https://en.wikisource.org/wiki/The_Adventures_of_Sherlock_Holmes/The_Red-Headed_League"
        format_type = "wikisource_html"

    results.append({
        "id": cid,
        "author": c["author"],
        "title": title,
        "language": lang,
        "original_url": url,
        "download_url": download_url,
        "format_type": format_type,
    })

print(f"Prepared plan for {len(results)} works.")
Path("corpus/curation/download_plan_73.json").write_text(json.dumps(results, ensure_ascii=False, indent=2), encoding="utf-8")
