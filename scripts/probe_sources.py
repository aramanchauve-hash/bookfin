import json
import time
import urllib.request
import urllib.error
from pathlib import Path

plan = json.loads(Path("corpus/curation/download_plan_73.json").read_text(encoding="utf-8"))

headers = {
    "User-Agent": "BookfinCuratedV1/1.0 (https://bookfin.app; contact@bookfin.app) Python/3.12"
}

probe_results = []
errors = []

print(f"Probing {len(plan)} URLs...")
for i, item in enumerate(plan, 1):
    cid = item["id"]
    url = item["download_url"]
    try:
        req = urllib.request.Request(url, headers=headers)
        # Using GET with small read instead of HEAD because some servers (Gutenberg/Wikisource) block or behave differently on HEAD
        with urllib.request.urlopen(req, timeout=12) as resp:
            status = resp.status
            final_url = resp.geturl()
            ctype = resp.headers.get("Content-Type", "")
            data_sample = resp.read(256)
            probe_results.append({
                "id": cid,
                "status": status,
                "final_url": final_url,
                "content_type": ctype,
                "ok": True,
            })
            print(f"[{i:02d}/73] OK {status}: {cid} ({ctype.split(';')[0]})")
    except Exception as e:
        print(f"[{i:02d}/73] ERROR {cid}: {e}")
        errors.append({"id": cid, "url": url, "error": str(e)})
        probe_results.append({
            "id": cid,
            "status": getattr(e, "code", 0),
            "error": str(e),
            "ok": False,
        })
    time.sleep(0.1)  # respectful delay

print(f"\nProbe finished. Success: {len(probe_results) - len(errors)}, Errors: {len(errors)}")
Path("corpus/curation/probe_results.json").write_text(json.dumps({"results": probe_results, "errors": errors}, ensure_ascii=False, indent=2), encoding="utf-8")
