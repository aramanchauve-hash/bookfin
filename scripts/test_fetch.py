import os
import json
import re
import urllib.request
import urllib.parse
from html.parser import HTMLParser

class CleanTextExtractor(HTMLParser):
    def __init__(self):
        super().__init__()
        self.text = []
        self.ignore_stack = []

    def handle_starttag(self, tag, attrs):
        attr_dict = dict(attrs)
        classes = set(attr_dict.get('class', '').split())
        ids = set(attr_dict.get('id', '').split())
        should_ignore = (
            tag in {'head', 'script', 'style', 'noscript', 'meta', 'link'}
            or bool(classes & {
                'ws-noexport', 'header', 'nav', 'mw-jump-link', 'noprint', 
                'ws-summary', 'navigation-box', 'sister-project', 'ws-dynamic-links',
                'catlinks', 'printfooter', 'metadata'
            })
            or bool(ids & {'toc', 'mw-navigation', 'siteNotice', 'mw-data-after-content'})
        )
        if self.ignore_stack or should_ignore:
            self.ignore_stack.append(tag)
        else:
            if tag in ('p', 'br', 'div', 'h1', 'h2', 'h3', 'h4', 'blockquote'):
                self.text.append('\n\n')

    def handle_endtag(self, tag):
        if self.ignore_stack:
            if self.ignore_stack[-1] == tag:
                self.ignore_stack.pop()
        else:
            if tag in ('p', 'div', 'h1', 'h2', 'h3', 'h4', 'blockquote'):
                self.text.append('\n\n')

    def handle_data(self, data):
        if not self.ignore_stack:
            self.text.append(data)

def fetch_wikisource(lang, page_title):
    endpoint = f"https://{lang}.wikisource.org/w/api.php"
    params = {
        'action': 'parse',
        'page': page_title,
        'prop': 'text',
        'format': 'json',
        'redirects': '1'
    }
    url = endpoint + '?' + urllib.parse.urlencode(params)
    req = urllib.request.Request(url, headers={'User-Agent': 'BookfinIngestBot/1.0 (academic; literary research)'})
    try:
        with urllib.request.urlopen(req, timeout=15) as resp:
            data = json.loads(resp.read().decode('utf-8'))
            if 'error' in data:
                return None, f"API Error: {data['error']}"
            html = data.get('parse', {}).get('text', {}).get('*', '')
            actual_title = data.get('parse', {}).get('title', page_title)
            
            parser = CleanTextExtractor()
            parser.feed(html)
            raw = ''.join(parser.text)
            
            # Clean typography
            cleaned = re.sub(r'[ \t\r\f\v]+', ' ', raw)
            cleaned = re.sub(r'\[\d+\]', '', cleaned)
            cleaned = re.sub(r'\n{3,}', '\n\n', cleaned).strip()
            return (actual_title, cleaned), None
    except Exception as e:
        return None, str(e)

if __name__ == '__main__':
    stories = [
        # FR
        ("fr", "Contes du jour et de la nuit (éd. Flammarion, 1885)/La Parure", "La Parure", "Guy de Maupassant", 1885),
        ("fr", "Miss Harriet (recueil)/La Ficelle", "La Ficelle", "Guy de Maupassant", 1884),
        ("fr", "Mademoiselle Fifi (recueil, 1883)/Deux amis", "Deux amis", "Guy de Maupassant", 1883),
        ("fr", "Contes de la bécasse/Menuet", "Menuet", "Guy de Maupassant", 1882),
        ("fr", "Mademoiselle Fifi (recueil, 1883)/La Folle", "La Folle", "Guy de Maupassant", 1882),
        ("fr", "Le Horla (recueil, 1887)/Le Horla", "Le Horla", "Guy de Maupassant", 1887),
        ("fr", "Lettres de mon moulin/La Chèvre de M. Seguin", "La Chèvre de M. Seguin", "Alphonse Daudet", 1869),
        ("fr", "Lettres de mon moulin/Le Secret de maître Cornille", "Le Secret de maître Cornille", "Alphonse Daudet", 1869),
        ("fr", "Une passion dans le désert", "Une passion dans le désert", "Honoré de Balzac", 1830),
        ("fr", "Mosaïque (Mérimée)/Mateo Falcone", "Mateo Falcone", "Prosper Mérimée", 1829),
        ("fr", "Le Spleen de Paris/L’Étranger", "L'Étranger", "Charles Baudelaire", 1869),
        ("fr", "Le Spleen de Paris/Le Désespoir de la vieille", "Le Désespoir de la vieille", "Charles Baudelaire", 1869),
        # EN
        ("en", "The Works of the Edgar Allan Poe/Volume 2/The Tell-Tale Heart", "The Tell-Tale Heart", "Edgar Allan Poe", 1843),
        ("en", "The Cask of Amontillado", "The Cask of Amontillado", "Edgar Allan Poe", 1846),
        ("en", "The Oval Portrait", "The Oval Portrait", "Edgar Allan Poe", 1842),
        ("en", "The Gift of the Magi", "The Gift of the Magi", "O. Henry", 1905),
        ("en", "The Last Leaf", "The Last Leaf", "O. Henry", 1907),
        ("en", "The Adventures of Sherlock Holmes (1892)/A Scandal in Bohemia", "A Scandal in Bohemia", "Arthur Conan Doyle", 1891),
        ("en", "To Build a Fire (1908)", "To Build a Fire", "Jack London", 1908),
        ("en", "The Bet and Other Stories/The Bet", "The Bet", "Anton Chekhov", 1889),
        ("en", "The Schoolmistress and Other Stories/Misery", "Misery", "Anton Chekhov", 1886),
        ("en", "The Story of an Hour", "The Story of an Hour", "Kate Chopin", 1894),
        # ES
        ("es", "El almohadón de plumas", "El almohadón de plumas", "Horacio Quiroga", 1907),
        ("es", "A la deriva", "A la deriva", "Horacio Quiroga", 1912),
        ("es", "La insolación", "La insolación", "Horacio Quiroga", 1908),
    ]

    print(f"Testing fetch for {len(stories)} stories...")
    for lang, page, title, author, yr in stories:
        res, err = fetch_wikisource(lang, page)
        if err:
            print(f"[FAIL] ({lang}) {title} by {author}: {err}")
        else:
            actual_title, text = res
            print(f"[OK] ({lang}) {title} by {author} ({len(text)} chars)")
