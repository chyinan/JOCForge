import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent


def require(condition, message):
    if not condition:
        raise SystemExit(message)


manifest_path = ROOT / "MANIFEST.json"
manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
seal = json.loads((ROOT / "package_seal.json").read_text(encoding="utf-8"))
manifest_hash = hashlib.sha256(manifest_path.read_bytes()).hexdigest()
require(manifest.get("schema") == "wash-sanitized-supplement.v1", "manifest schema mismatch")
require(manifest.get("package_id") == "JOCFORGE-I0-D0-AU-COMPOSITION-BOUNDARY-V1", "package id mismatch")
require(seal.get("manifest_sha256") == manifest_hash, "seal does not bind manifest")
require((ROOT / "MANIFEST.sha256").read_text(encoding="utf-8").split()[0] == manifest_hash, "manifest checksum mismatch")

expected = {entry["path"] for entry in manifest.get("files", [])}
actual = {path.name for path in ROOT.iterdir() if path.is_file()} - {"MANIFEST.json", "MANIFEST.sha256", "package_seal.json"}
require(actual == expected, "payload set mismatch")
for entry in manifest["files"]:
    path = ROOT / entry["path"]
    require(path.stat().st_size == entry["bytes"], f"size mismatch: {entry['path']}")
    require(hashlib.sha256(path.read_bytes()).hexdigest() == entry["sha256"], f"hash mismatch: {entry['path']}")

vector = json.loads((ROOT / "TARGET-AU-ORDER-VECTOR.json").read_text(encoding="utf-8"))
au = vector["access_unit"]
require([item["role"] for item in au["ordered_substreams"]] == ["I0", "D0"], "AU order mismatch")
require([item["rate_kbps"] for item in au["ordered_substreams"]] == [640, 1024], "substream rates mismatch")
require(au["aggregate_rate_kbps"] == 1664, "aggregate rate mismatch")
require(au["single_1664_kbps_frame"] is False, "aggregate/frame distinction mismatch")
require(au["same_source_generation_required"] is True, "source generation requirement mismatch")
require(au["publish_complete_access_unit_atomically"] is True, "AU publication requirement mismatch")
require(au["partial_i0_only_publish"] == "REJECT", "partial publication rule mismatch")
require(vector["open_boundaries"]["d0_source_row_to_semantic_role"] == "UNBOUND", "D0 source-role boundary changed")
require(vector["open_boundaries"]["native_d0_audio_algorithm"] == "UNBOUND", "D0 audio boundary changed")
require(vector["open_boundaries"]["production_allowed"] is False, "production gate changed")
print("PASS: manifest, seal, I0/D0 target order, and Wash boundaries")
