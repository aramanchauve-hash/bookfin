import urllib.request
import re
from typing import List, Tuple

HEADERS = {'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36'}

def search_gutenberg(query: str) -> List[Tuple[str, str, str]]:
    url = f"https://www.gutenberg.org/ebooks/search/?query={urllib.parse.quote(query)}"
    req = urllib.request.Request(url, headers=HEADERS)
    try:
        with urllib.request.urlopen(req, timeout=15) as resp:
            html = resp.read().decode('utf-8', errors='replace')
    except Exception as e:
        print(f"Error searching for {query}: {e}")
        return []
        
    results = []
    # Pattern to find ebook entries
    pattern = re.compile(r'<li class="booklink">.*?<a class="link" href="/ebooks/(\d+)".*?<span class="title">([^<]+)</span>.*?<span class="subtitle">([^<]*)</span>', re.DOTALL)
    for m in pattern.finditer(html):
        mid, title, author = m.groups()
        results.append((mid.strip(), title.strip(), author.strip()))
    return results

if __name__ == "__main__":
    import urllib.parse
    queries = [
        "A Sentimental Journey Laurence Sterne",
        "Table Talk William Hazlitt",
        "The Private Papers of Henry Ryecroft George Gissing",
        "Le Journal d'une femme de chambre Octave Mirbeau",
        "Le Diable boiteux Alain-Rene Lesage",
        "Carmen Prosper Merimee",
        "El doncel de don Enrique el Doliente Mariano Jose de Larra",
        "Sonata de otono Ramon del Valle-Inclan",
    ]
    for q in queries:
        print(f"\n=== Query: {q} ===")
        res = search_gutenberg(q)
        for mid, title, author in res[:5]:
            print(f"  PG #{mid} | Title: {title} | Author: {author}")

