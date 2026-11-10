import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent


def require(ok, message):
    if not ok:
        raise SystemExit(message)


manifest_path = ROOT / "MANIFEST.json"
manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
seal = json.loads((ROOT / "package_seal.json").read_text(encoding="utf-8"))
manifest_hash = hashlib.sha256(manifest_path.read_bytes()).hexdigest()
require(manifest["package_id"] == "JOCFORGE-ATMOS71-I0-FRAME-SYNTAX-GAP-V1", "package id mismatch")
require(seal["manifest_sha256"] == manifest_hash, "seal binding mismatch")
require((ROOT / "MANIFEST.sha256").read_text(encoding="utf-8").split()[0] == manifest_hash, "manifest checksum mismatch")
expected = {entry["path"] for entry in manifest["files"]}
actual = {p.name for p in ROOT.iterdir() if p.is_file()} - {"MANIFEST.json", "MANIFEST.sha256", "package_seal.json"}
require(actual == expected, "payload set mismatch")
for entry in manifest["files"]:
    p = ROOT / entry["path"]
    require(p.stat().st_size == entry["bytes"], f"size mismatch: {entry['path']}")
    require(hashlib.sha256(p.read_bytes()).hexdigest() == entry["sha256"], f"hash mismatch: {entry['path']}")

data = json.loads((ROOT / "I0-SYNTAX-BOUNDARY.json").read_text(encoding="utf-8"))
require(data["official_structure_receipt"]["classification"] == "legacy_independent", "official class mismatch")
require(data["official_structure_receipt"]["exact_bsid"] == "NOT_CAPTURED", "official BSID boundary changed")
require(data["isolated_selected_prefix_source"]["syntax"] == "E-AC-3 BSID 16; strmtyp=0", "isolated prefix mismatch")
require(data["isolated_selected_prefix_source"]["implementation_equivalence_to_official_i0"] == "UNBOUND", "I0 equivalence boundary changed")
require(data["profile_boundaries"]["new_7_1_profile_production_allowed"] is False, "production gate changed")
print("PASS: I0 syntax gap, source hashes, and production boundary")
