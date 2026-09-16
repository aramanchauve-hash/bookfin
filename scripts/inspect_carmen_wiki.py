import urllib.request
import re

urls = [
    'https://fr.wikisource.org/wiki/Carmen_(M%C3%A9rim%C3%A9e)',
    'https://fr.wikisource.org/wiki/Carmen_(Nouvelle)',
]

for u in urls:
    req = urllib.request.Request(u, headers={'User-Agent': 'Mozilla/5.0'})
    try:
        with urllib.request.urlopen(req, timeout=15) as resp:
            content = resp.read().decode('utf-8', errors='replace')
            print(f"\nURL: {u} -> Length: {len(content):,} chars")
            links = re.findall(r'href="(/wiki/[^"]+)"', content)
            sub_links = [l for l in links if 'Carmen' in l]
            for l in sorted(set(sub_links))[:10]:
                print(f"  {l}")
    except Exception as e:
        print(f"Error {u}: {e}")

