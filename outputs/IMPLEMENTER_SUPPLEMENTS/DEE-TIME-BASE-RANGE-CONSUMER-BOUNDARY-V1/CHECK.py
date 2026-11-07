from __future__ import annotations
import hashlib
import json
from pathlib import Path

root = Path(__file__).resolve().parent
manifest = json.loads((root / "MANIFEST.json").read_text(encoding="utf-8"))
seal = json.loads((root / "package_seal.json").read_text(encoding="utf-8"))
assert manifest["schema"] == "jocforge.wash_supplement_manifest.v1"
assert manifest["package_id"] == seal["package_id"]
for payload in manifest["payloads"]:
    data = (root / payload["file"]).read_bytes()
    assert len(data) == payload["bytes"]
    assert hashlib.sha256(data).hexdigest() == payload["sha256"]
assert seal["manifest_sha256"] == hashlib.sha256((root / "MANIFEST.json").read_bytes()).hexdigest()
assert seal["payload_file_count"] == len(manifest["payloads"])
assert seal["payload_hash_audit"] == f"PASS_{len(manifest['payloads'])}_OF_{len(manifest['payloads'])}"
assert seal["checker_status"] == "PASS_NORMAL_AND_PYTHON_O"
assert seal["status"] == manifest["status"]
vectors = json.loads((root / "INPUT-ONLY-VECTORS.json").read_text(encoding="utf-8"))
assert [item["token"] for item in vectors["vectors"]] == ["file_position", "embedded_timecode"]
assert all(item["status"] == "PROPOSED_NOT_RUN" and item["expected_output"] is None for item in vectors["vectors"])
print("TIMEBASE_SUPPLEMENT_CHECK=PASS")