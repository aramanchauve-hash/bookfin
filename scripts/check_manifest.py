import json

with open('corpus/pilot_short_stories/manifest.json', 'r', encoding='utf-8') as f:
    m = json.load(f)

print(f"Total stories: {len(m['stories'])}")
for i, s in enumerate(m['stories'], 1):
    print(f"{i:2d}. [{s['language_tag']}] \"{s['title']}\" par {s['author']} ({s['publication_year']}) : {s['char_count']} chars")
