import json
from pathlib import Path

manifest = json.loads(Path("corpus/curation/library_v1_candidates.json").read_text(encoding="utf-8"))
accepted = [c for c in manifest["candidates"] if c["status"] == "ACCEPTED"]

print(f"Total accepted: {len(accepted)}")
by_provider = {}
for c in accepted:
    p = c["source_provider"]
    by_provider.setdefault(p, []).append(c)

for p, items in by_provider.items():
    print(f"\n--- Provider: {p} ({len(items)} items) ---")
    for it in items[:5]:
        print(f"  [{it['id']}] {it['title']} -> {it['source_url']}")
    if len(items) > 5:
        print(f"  ... and {len(items)-5} more")

