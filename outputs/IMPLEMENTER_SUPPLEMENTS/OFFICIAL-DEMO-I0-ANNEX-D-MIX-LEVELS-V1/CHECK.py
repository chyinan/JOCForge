from __future__ import annotations

import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "OFFICIAL-DEMO-I0-ANNEX-D-MIX-LEVELS-V1"
SOURCE_SHA256 = "f976aba7a4750be80b4373a5d7fabf2ad101e0c227fc4bd77716b76879d546cd"
EXPECTED = {
    "xbsi1e": (1, 1, 74, 75, 43466, 43467),
    "dmixmod": (2, 1, 75, 77, 43467, 43469),
    "ltrtcmixlev": (3, 4, 77, 80, 43469, 43472),
    "ltrtsurmixlev": (3, 4, 80, 83, 43472, 43475),
    "lorocmixlev": (3, 4, 83, 86, 43475, 43478),
    "lorosurmixlev": (3, 4, 86, 89, 43478, 43481),
    "xbsi2e": (1, 1, 89, 90, 43481, 43482),
}

def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(f"FAIL {message}")

def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()

manifest_raw = (ROOT / "MANIFEST.json").read_bytes()
manifest_hash = digest(manifest_raw)
manifest = json.loads(manifest_raw)
require(manifest.get("package_id") == PACKAGE_ID, "package id")
require((ROOT / "MANIFEST.sha256").read_text(encoding="ascii").strip() == f"{manifest_hash} MANIFEST.json", "manifest digest")
files = manifest.get("files")
require(isinstance(files, list) and {item.get("path") for item in files} == {"FIELD-VALUES.json", "SOURCE-RECEIPTS.json", "REPORT.md", "CHECK.py"}, "exact payload inventory")
for item in files:
    path = ROOT / item["path"]
    data = path.read_bytes()
    require(len(data) == item["bytes"] and digest(data) == item["sha256"], f"payload hash {item['path']}")
seal = json.loads((ROOT / "package_seal.json").read_text(encoding="utf-8"))
require(seal.get("manifest_sha256") == manifest_hash and seal.get("files") == files, "seal")
values = json.loads((ROOT / "FIELD-VALUES.json").read_text(encoding="utf-8"))
require(values["sample"]["source_sha256"] == SOURCE_SHA256, "source identity")
require(values["sample"]["aggregate_rate_kbps"] == 1280, "sample rate scope")
require(values["sample"]["i0_header"]["bsid"] == 6 and values["sample"]["i0_header"]["acmod"] == 7 and values["sample"]["i0_header"]["lfeon"] == 1, "I0 identity")
fields = {item["name"]: item for item in values["annex_d_bsi"]["fields"]}
require(set(fields) == set(EXPECTED), "field set")
for name, (width, value, frame_start, frame_end, file_start, file_end) in EXPECTED.items():
    item = fields[name]
    require(item["width_bits"] == width and item["value"] == value, f"{name} value")
    require(item["frame_bits"] == [frame_start, frame_end] and item["file_bits"] == [file_start, file_end], f"{name} offsets")
require(values["annex_d_bsi"]["xbsi2e_following_14_bits"] == "NOT_CAPTURED", "xbsi2e tail status")
require(values["user_target_transfer"]["target_aggregate_rate_kbps"] == 1664 and values["user_target_transfer"]["target_i0_rate_kbps"] == 640 and values["user_target_transfer"]["target_d0_rate_kbps"] == 1024, "target rate contract")
require(values["user_target_transfer"]["target_is_observed_output"] is False, "sample/target separation")
require(values["remaining_gates"]["complete_1664_kbps_output"] == "NOT_RUN" and values["remaining_gates"]["d0_audio_algorithm"] == "UNBOUND", "open gates")
provenance = json.loads((ROOT / "SOURCE-RECEIPTS.json").read_text(encoding="utf-8"))
require(provenance["source"]["sha256"] == SOURCE_SHA256 and provenance["source"]["source_path_included"] is False, "sanitized provenance")
require(provenance["verification"]["parser_replay_field_ledger_byte_identical"] is True, "parser replay receipt")
print(f"PASS package={PACKAGE_ID} manifest={manifest_hash}")
print("PASS observed I0 Annex-D values, source identity, target contract, and NOT_CAPTURED follow-up fields")
