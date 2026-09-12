import os
import json
import re
import time
import uuid
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
                'catlinks', 'printfooter', 'metadata', 'ws-header', 'ws-footer',
                'pr_page_header', 'pr_page_footer', 'pagenum'
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
    req = urllib.request.Request(url, headers={'User-Agent': 'BookfinPilotCorpus/1.0 (https://github.com/bookfin; contact@bookfin.org)'})
    try:
        with urllib.request.urlopen(req, timeout=25) as resp:
            data = json.loads(resp.read().decode('utf-8'))
            if 'error' in data:
                return None, f"API Error: {data['error']}"
            html = data.get('parse', {}).get('text', {}).get('*', '')
            actual_title = data.get('parse', {}).get('title', page_title)
            
            parser = CleanTextExtractor()
            parser.feed(html)
            raw = ''.join(parser.text)
            
            # Clean typography & normalise spaces
            cleaned = re.sub(r'[ \t\r\f\v]+', ' ', raw)
            # Remove isolated editor footnote markers like [1], [2], [p. 12]
            cleaned = re.sub(r'\[\d+\]', '', cleaned)
            cleaned = re.sub(r'\[p\.\s*\d+\]', '', cleaned, flags=re.IGNORECASE)
            # Normalise multiple line breaks
            cleaned = re.sub(r'\n{3,}', '\n\n', cleaned).strip()
            return (actual_title, cleaned), None
    except Exception as e:
        return None, str(e)

STORIES = [
    # -------------------------------------------------------------
    # FRANÇAIS (12 nouvelles)
    # -------------------------------------------------------------
    {
        "slug": "fr_maupassant_la_parure",
        "title": "La Parure",
        "author": "Guy de Maupassant",
        "lang": "fr",
        "year": 1885,
        "wikisource_lang": "fr",
        "wikisource_page": "Contes du jour et de la nuit (éd. Flammarion, 1885)/La Parure",
        "source_name": "Wikisource (Contes du jour et de la nuit, Flammarion, 1885)",
        "source_url": "https://fr.wikisource.org/wiki/Contes_du_jour_et_de_la_nuit_(%C3%A9d._Flammarion,_1885)/La_Parure",
    },
    {
        "slug": "fr_maupassant_la_ficelle",
        "title": "La Ficelle",
        "author": "Guy de Maupassant",
        "lang": "fr",
        "year": 1884,
        "wikisource_lang": "fr",
        "wikisource_page": "Miss Harriet (recueil)/La Ficelle",
        "source_name": "Wikisource (Miss Harriet, Victor Havard, 1884)",
        "source_url": "https://fr.wikisource.org/wiki/Miss_Harriet_(recueil)/La_Ficelle",
    },
    {
        "slug": "fr_maupassant_deux_amis",
        "title": "Deux amis",
        "author": "Guy de Maupassant",
        "lang": "fr",
        "year": 1883,
        "wikisource_lang": "fr",
        "wikisource_page": "Mademoiselle Fifi (recueil, Ollendorff 1898)/Deux amis",
        "source_name": "Wikisource (Mademoiselle Fifi, Ollendorff, 1898)",
        "source_url": "https://fr.wikisource.org/wiki/Mademoiselle_Fifi_(recueil,_Ollendorff_1898)/Deux_amis",
    },
    {
        "slug": "fr_maupassant_menuet",
        "title": "Menuet",
        "author": "Guy de Maupassant",
        "lang": "fr",
        "year": 1882,
        "wikisource_lang": "fr",
        "wikisource_page": "Contes de la bécasse/Menuet",
        "source_name": "Wikisource (Contes de la bécasse, Rouveyre et G. Blond, 1883)",
        "source_url": "https://fr.wikisource.org/wiki/Contes_de_la_b%C3%A9casse/Menuet",
    },
    {
        "slug": "fr_maupassant_la_folle",
        "title": "La Folle",
        "author": "Guy de Maupassant",
        "lang": "fr",
        "year": 1882,
        "wikisource_lang": "fr",
        "wikisource_page": "Contes de la bécasse/La Folle",
        "source_name": "Wikisource (Contes de la bécasse, Rouveyre et G. Blond, 1883)",
        "source_url": "https://fr.wikisource.org/wiki/Contes_de_la_b%C3%A9casse/La_Folle",
    },
    {
        "slug": "fr_maupassant_le_horla",
        "title": "Le Horla",
        "author": "Guy de Maupassant",
        "lang": "fr",
        "year": 1887,
        "wikisource_lang": "fr",
        "wikisource_page": "Le Horla (recueil, Ollendorff 1895)/Le Horla",
        "source_name": "Wikisource (Le Horla, Paul Ollendorff, 1895)",
        "source_url": "https://fr.wikisource.org/wiki/Le_Horla_(recueil,_Ollendorff_1895)/Le_Horla",
    },
    {
        "slug": "fr_daudet_chevre_monsieur_seguin",
        "title": "La chèvre de monsieur Seguin",
        "author": "Alphonse Daudet",
        "lang": "fr",
        "year": 1869,
        "wikisource_lang": "fr",
        "wikisource_page": "Lettres de mon moulin/La chèvre de monsieur Seguin",
        "source_name": "Wikisource (Lettres de mon moulin, J. Hetzel, 1869)",
        "source_url": "https://fr.wikisource.org/wiki/Lettres_de_mon_moulin/La_ch%C3%A8vre_de_monsieur_Seguin",
    },
    {
        "slug": "fr_daudet_secret_cornille",
        "title": "Le secret de maître Cornille",
        "author": "Alphonse Daudet",
        "lang": "fr",
        "year": 1869,
        "wikisource_lang": "fr",
        "wikisource_page": "Lettres de mon moulin/Le secret de maître Cornille",
        "source_name": "Wikisource (Lettres de mon moulin, J. Hetzel, 1869)",
        "source_url": "https://fr.wikisource.org/wiki/Lettres_de_mon_moulin/Le_secret_de_ma%C3%AEtre_Cornille",
    },
    {
        "slug": "fr_daudet_messes_basses",
        "title": "Les trois messes basses",
        "author": "Alphonse Daudet",
        "lang": "fr",
        "year": 1869,
        "wikisource_lang": "fr",
        "wikisource_page": "Lettres de mon moulin/Les trois messes basses",
        "source_name": "Wikisource (Lettres de mon moulin, J. Hetzel, 1869)",
        "source_url": "https://fr.wikisource.org/wiki/Lettres_de_mon_moulin/Les_trois_messes_basses",
    },
    {
        "slug": "fr_balzac_passion_desert",
        "title": "Une passion dans le désert",
        "author": "Honoré de Balzac",
        "lang": "fr",
        "year": 1830,
        "wikisource_lang": "fr",
        "wikisource_page": "Une passion dans le désert",
        "source_name": "Wikisource (Revue de Paris, 1830 / Études philosophiques)",
        "source_url": "https://fr.wikisource.org/wiki/Une_passion_dans_le_d%C3%A9sert",
    },
    {
        "slug": "fr_merimee_mateo_falcone",
        "title": "Mateo Falcone",
        "author": "Prosper Mérimée",
        "lang": "fr",
        "year": 1829,
        "wikisource_lang": "fr",
        "wikisource_page": "Mateo Falcone",
        "source_name": "Wikisource (Revue de Paris, 1829 / Mosaïque, 1833)",
        "source_url": "https://fr.wikisource.org/wiki/Mateo_Falcone",
    },
    {
        "slug": "fr_baudelaire_enivrez_vous",
        "title": "Enivrez-vous",
        "author": "Charles Baudelaire",
        "lang": "fr",
        "year": 1869,
        "wikisource_lang": "fr",
        "wikisource_page": "Enivrez-vous",
        "source_name": "Wikisource (Le Spleen de Paris, Petits poèmes en prose, Michel Lévy, 1869)",
        "source_url": "https://fr.wikisource.org/wiki/Enivrez-vous",
    },

    # -------------------------------------------------------------
    # ANGLAIS (10 nouvelles)
    # -------------------------------------------------------------
    {
        "slug": "en_poe_tell_tale_heart",
        "title": "The Tell-Tale Heart",
        "author": "Edgar Allan Poe",
        "lang": "en",
        "year": 1843,
        "wikisource_lang": "en",
        "wikisource_page": "Mystery Tales of Edgar Allan Poe/The Tell-Tale Heart",
        "source_name": "Wikisource (Mystery Tales of Edgar Allan Poe, 1907)",
        "source_url": "https://en.wikisource.org/wiki/Mystery_Tales_of_Edgar_Allan_Poe/The_Tell-Tale_Heart",
    },
    {
        "slug": "en_poe_cask_amontillado",
        "title": "The Cask of Amontillado",
        "author": "Edgar Allan Poe",
        "lang": "en",
        "year": 1846,
        "wikisource_lang": "en",
        "wikisource_page": "The Works of the Late Edgar Allan Poe (1850)/Volume 1/The Cask of Amontillado",
        "source_name": "Wikisource (The Works of the Late Edgar Allan Poe, Redfield, 1850)",
        "source_url": "https://en.wikisource.org/wiki/The_Works_of_the_Late_Edgar_Allan_Poe_(1850)/Volume_1/The_Cask_of_Amontillado",
    },
    {
        "slug": "en_poe_oval_portrait",
        "title": "The Oval Portrait",
        "author": "Edgar Allan Poe",
        "lang": "en",
        "year": 1842,
        "wikisource_lang": "en",
        "wikisource_page": "The Works of the Late Edgar Allan Poe (1850)/Volume 1/The Oval Portrait",
        "source_name": "Wikisource (The Works of the Late Edgar Allan Poe, Redfield, 1850)",
        "source_url": "https://en.wikisource.org/wiki/The_Works_of_the_Late_Edgar_Allan_Poe_(1850)/Volume_1/The_Oval_Portrait",
    },
    {
        "slug": "en_henry_gift_magi",
        "title": "The Gift of the Magi",
        "author": "O. Henry",
        "lang": "en",
        "year": 1905,
        "wikisource_lang": "en",
        "wikisource_page": "The Gift of the Magi",
        "source_name": "Wikisource (The Four Million, Doubleday, Page & Co., 1906)",
        "source_url": "https://en.wikisource.org/wiki/The_Gift_of_the_Magi",
    },
    {
        "slug": "en_henry_last_leaf",
        "title": "The Last Leaf",
        "author": "O. Henry",
        "lang": "en",
        "year": 1907,
        "wikisource_lang": "en",
        "wikisource_page": "The Last Leaf (Henry)",
        "source_name": "Wikisource (The Trimmed Lamp, Doubleday, Page & Co., 1907)",
        "source_url": "https://en.wikisource.org/wiki/The_Last_Leaf_(Henry)",
    },
    {
        "slug": "en_doyle_scandal_bohemia",
        "title": "A Scandal in Bohemia",
        "author": "Arthur Conan Doyle",
        "lang": "en",
        "year": 1891,
        "wikisource_lang": "en",
        "wikisource_page": "The Adventures of Sherlock Holmes/A Scandal in Bohemia",
        "source_name": "Wikisource (The Adventures of Sherlock Holmes, George Newnes, 1892)",
        "source_url": "https://en.wikisource.org/wiki/The_Adventures_of_Sherlock_Holmes/A_Scandal_in_Bohemia",
    },
    {
        "slug": "en_doyle_red_headed_league",
        "title": "The Red-Headed League",
        "author": "Arthur Conan Doyle",
        "lang": "en",
        "year": 1891,
        "wikisource_lang": "en",
        "wikisource_page": "The Adventures of Sherlock Holmes/The Red-Headed League",
        "source_name": "Wikisource (The Adventures of Sherlock Holmes, George Newnes, 1892)",
        "source_url": "https://en.wikisource.org/wiki/The_Adventures_of_Sherlock_Holmes/The_Red-Headed_League",
    },
    {
        "slug": "en_london_build_fire",
        "title": "To Build a Fire",
        "author": "Jack London",
        "lang": "en",
        "year": 1908,
        "wikisource_lang": "en",
        "wikisource_page": "Century Magazine/Volume 76/Issue 4/To Build a Fire",
        "source_name": "Wikisource (Century Magazine, Aug 1908)",
        "source_url": "https://en.wikisource.org/wiki/Century_Magazine/Volume_76/Issue_4/To_Build_a_Fire",
    },
    {
        "slug": "en_saki_open_window",
        "title": "The Open Window",
        "author": "Saki (H.H. Munro)",
        "lang": "en",
        "year": 1914,
        "wikisource_lang": "en",
        "wikisource_page": "The Open Window (Saki)",
        "source_name": "Wikisource (Beasts and Super-Beasts, John Lane, 1914)",
        "source_url": "https://en.wikisource.org/wiki/The_Open_Window_(Saki)",
    },
    {
        "slug": "en_chopin_story_hour",
        "title": "The Story of an Hour",
        "author": "Kate Chopin",
        "lang": "en",
        "year": 1894,
        "wikisource_lang": "en",
        "wikisource_page": "The Story of an Hour",
        "source_name": "Wikisource (Vogue, Dec 1894)",
        "source_url": "https://en.wikisource.org/wiki/The_Story_of_an_Hour",
    },

    # -------------------------------------------------------------
    # ESPAGNOL (3 nouvelles)
    # -------------------------------------------------------------
    {
        "slug": "es_quiroga_almohadon_plumas",
        "title": "El almohadón de plumas",
        "author": "Horacio Quiroga",
        "lang": "es",
        "year": 1907,
        "wikisource_lang": "es",
        "wikisource_page": "El almohadón de pluma",
        "source_name": "Wikisource (Caras y Caretas, 1907 / Cuentos de amor de locura y de muerte, 1917)",
        "source_url": "https://es.wikisource.org/wiki/El_almohad%C3%B3n_de_pluma",
    },
    {
        "slug": "es_quiroga_a_la_deriva",
        "title": "A la deriva",
        "author": "Horacio Quiroga",
        "lang": "es",
        "year": 1912,
        "wikisource_lang": "es",
        "wikisource_page": "A la deriva",
        "source_name": "Wikisource (Fray Mocho, 1912 / Cuentos de amor de locura y de muerte, 1917)",
        "source_url": "https://es.wikisource.org/wiki/A_la_deriva",
    },
    {
        "slug": "es_quiroga_insolacion",
        "title": "La insolación",
        "author": "Horacio Quiroga",
        "lang": "es",
        "year": 1908,
        "wikisource_lang": "es",
        "wikisource_page": "La insolación",
        "source_name": "Wikisource (Caras y Caretas, 1908 / Cuentos de amor de locura y de muerte, 1917)",
        "source_url": "https://es.wikisource.org/wiki/La_insolaci%C3%B3n",
    },
]

NAMESPACE = uuid.UUID('7b1981a4-6842-4dc8-a83a-867df3c965e9')

def main():
    base_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    texts_dir = os.path.join(base_dir, 'corpus', 'pilot_short_stories', 'texts')
    manifest_path = os.path.join(base_dir, 'corpus', 'pilot_short_stories', 'manifest.json')
    os.makedirs(texts_dir, exist_ok=True)

    manifest_entries = []
    print(f"=== TÉLÉCHARGEMENT DU CORPUS PILOTE ({len(STORIES)} NOUVELLES) ===")

    for i, s in enumerate(STORIES, 1):
        slug = s['slug']
        txt_path = os.path.join(texts_dir, f"{slug}.txt")
        rel_txt_path = f"corpus/pilot_short_stories/texts/{slug}.txt"

        print(f"[{i}/{len(STORIES)}] ({s['lang']}) {s['title']} par {s['author']}...", end=" ", flush=True)

        # Force re-download if file is suspiciously small (< 1500 chars) unless it's known to be short like Baudelaire
        needs_download = True
        if os.path.exists(txt_path):
            with open(txt_path, 'r', encoding='utf-8') as f:
                existing_text = f.read()
            if len(existing_text) > 800:
                print(f"[CACHE] ({len(existing_text)} car.)")
                needs_download = False
                text = existing_text

        if needs_download:
            time.sleep(1.5)
            res, err = fetch_wikisource(s['wikisource_lang'], s['wikisource_page'])
            if err or not res or len(res[1]) < 200:
                print(f"[ERREUR] {err or 'Texte vide'}")
                continue
            actual_title, text = res
            with open(txt_path, 'w', encoding='utf-8') as f:
                f.write(text)
            print(f"[OK] ({len(text)} car.)")

        work_id = str(uuid.uuid5(NAMESPACE, f"work_{slug}"))
        edition_id = str(uuid.uuid5(NAMESPACE, f"edition_{slug}"))

        manifest_entries.append({
            "work_id": work_id,
            "edition_id": edition_id,
            "slug": slug,
            "title": s['title'],
            "author": s['author'],
            "original_language_tag": s['lang'],
            "edition_title": s['title'],
            "language_tag": s['lang'],
            "publication_year": s['year'],
            "source_name": s['source_name'],
            "source_url": s['source_url'],
            "rights_status": "public_domain",
            "license": "public_domain",
            "file_path": rel_txt_path,
            "char_count": len(text)
        })

    with open(manifest_path, 'w', encoding='utf-8') as f:
        json.dump({
            "corpus_name": "Bookfin Pilot Short Stories",
            "version": 1,
            "generated_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
            "stories_count": len(manifest_entries),
            "stories": manifest_entries
        }, f, indent=2, ensure_ascii=False)

    print(f"\nManifeste écrit dans {manifest_path} avec {len(manifest_entries)} nouvelles.")

if __name__ == '__main__':
    main()
