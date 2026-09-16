import urllib.request
import re
from typing import Dict, Any

HEADERS = {'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36'}

def probe_url(url: str, check_tokens: list[str]) -> Dict[str, Any]:
    req = urllib.request.Request(url, headers=HEADERS)
    try:
        with urllib.request.urlopen(req, timeout=20) as resp:
            content = resp.read().decode('utf-8', errors='replace')
    except Exception as e:
        return {'url': url, 'error': str(e)}
        
    title_m = re.search(r'<title>(.*?)</title>', content, re.I | re.DOTALL)
    title = re.sub(r'\s+', ' ', title_m.group(1)).strip() if title_m else 'No title'
    
    token_results = {t: (t.lower() in content.lower()) for t in check_tokens}
    
    return {
        'url': url,
        'title': title,
        'bytes': len(content.encode('utf-8')),
        'chars': len(content),
        'tokens': token_results,
        'sample_start': content[5000:6000].replace('\n', ' ')[:200],
    }

if __name__ == "__main__":
    import urllib.parse
    tests_wiki = [
        "https://fr.wikisource.org/wiki/Carmen/Texte_entier",
        "https://fr.wikisource.org/wiki/Carmen",
        "https://fr.wikisource.org/wiki/Boule_de_suif/Texte_entier",
        "https://fr.wikisource.org/wiki/Boule_de_suif",
        "https://fr.wikisource.org/wiki/Le_Diable_boiteux/Texte_entier",
        "https://es.wikisource.org/wiki/" + urllib.parse.quote("Sonata_de_otoño"),
        "https://es.wikisource.org/wiki/" + urllib.parse.quote("El_doncel_de_don_Enrique_el_Doliente"),
        "https://www.gutenberg.org/cache/epub/53587/pg53587-images.html",
        "https://www.gutenberg.org/cache/epub/53588/pg53588-images.html",
        "https://www.gutenberg.org/cache/epub/53589/pg53589-images.html",
        "https://www.gutenberg.org/cache/epub/53590/pg53590-images.html",
        "https://www.gutenberg.org/cache/epub/35019/pg35019-images.html",
        "https://www.gutenberg.org/cache/epub/44142/pg44142-images.html",
        "https://www.gutenberg.org/cache/epub/2833/pg2833-images.html",
        "https://www.gutenberg.org/cache/epub/2834/pg2834-images.html",
    ]
    for url in tests_wiki:
        req = urllib.request.Request(url, headers=HEADERS)
        try:
            with urllib.request.urlopen(req, timeout=10) as resp:
                data = resp.read()
                print(f"OK ({len(data):,} bytes): {url}")
        except Exception as e:
            print(f"ERR ({e}): {url}")
