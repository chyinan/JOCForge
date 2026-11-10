"""Check the integrity and exact semantic scope of this Wash supplement."""
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent
EXPECTED_ID = "JOCFORGE-FLAT7X-1024-PARTIAL-STRUCTURE-V1"


def require(condition, message):
    if not condition:
        raise SystemExit(f"FAIL: {message}")


manifest_path = ROOT / "MANIFEST.json"
seal_path = ROOT / "package_seal.json"
manifest_bytes = manifest_path.read_bytes()
manifest_sha = hashlib.sha256(manifest_bytes).hexdigest()
manifest = json.loads(manifest_bytes)
seal = json.loads(seal_path.read_text(encoding="utf-8"))

require(manifest.get("package_id") == EXPECTED_ID, "package id")
require(manifest.get("status") == "PARTIAL_IMPLEMENTATION_READY", "partial-scope status")
require(seal.get("package_id") == EXPECTED_ID, "seal package id")
require(seal.get("manifest_sha256") == manifest_sha, "manifest seal")

for entry in manifest.get("files", []):
    path = ROOT / entry["path"]
    require(path.is_file(), f"missing payload {entry['path']}")
    data = path.read_bytes()
    require(len(data) == entry["bytes"], f"size mismatch {entry['path']}")
    require(hashlib.sha256(data).hexdigest() == entry["sha256"], f"hash mismatch {entry['path']}")

profile = json.loads((ROOT / "PROFILE-DELTA.json").read_text(encoding="utf-8"))
vector = json.loads((ROOT / "INPUT-ONLY-VECTORS.json").read_text(encoding="utf-8"))
gtc = json.loads((ROOT / "GTC-OWNER-PROJECTION.json").read_text(encoding="utf-8"))
require(profile["added_dimension"]["wire_field"] == "joc_dmx_config_idx", "wire field")
require(profile["added_dimension"]["wire_width_bits"] == 3, "wire width")
require(profile["added_dimension"]["wire_value"] == 1, "flat7x index")
require(profile["added_dimension"]["variant"] == "Flat7x", "layout variant")
require(profile["baseline"]["core_layout"] == "5.1", "base layout")
require(profile["baseline"]["core_rate_kbps"] == 1024, "base rate")
require(profile["production_profile_delta"]["keep_existing_5_1_row"] is True, "baseline preservation")
require(profile["production_profile_delta"]["add_mutable_state"] is False, "no new mutable state")
require(gtc["new_mutable_owner"] is False, "owner projection")
cases = {case["id"]: case for case in vector["vectors"]}
require(cases["flat7x-index1"]["expected"]["joc_dmx_config_idx"]["value"] == 1, "flat7x vector")
require(cases["reserved-index-rejection"]["expected"]["profile_resolution"] == "UNSUPPORTED_PROFILE", "reserved index vector")
print("PASS: package integrity, ETSI field crosswalk, separate layout dimension, immutable owner projection, and 1024/5.1 additive scope verified")
