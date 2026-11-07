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
    path = root / payload["file"]
    data = path.read_bytes()
    assert len(data) == payload["bytes"]
    assert hashlib.sha256(data).hexdigest() == payload["sha256"]
manifest_hash = hashlib.sha256((root / "MANIFEST.json").read_bytes()).hexdigest()
assert seal["manifest_sha256"] == manifest_hash
assert seal["payload_file_count"] == len(manifest["payloads"])
assert seal["payload_hash_audit"] == f"PASS_{len(manifest['payloads'])}_OF_{len(manifest['payloads'])}"
assert seal["checker_status"] == "PASS_NORMAL_AND_PYTHON_O"
assert seal["status"] == manifest["status"]
print("XML_BOUNDARY_SUPPLEMENT_CHECK=PASS")