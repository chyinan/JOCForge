from __future__ import annotations

import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "OFFICIAL-DEMO-I0-ANNEX-D-BSI-CONTINUATION-V2"
SOURCE_SHA256 = "f976aba7a4750be80b4373a5d7fabf2ad101e0c227fc4bd77716b76879d546cd"
EXPECTED = [
    ("syncword", 16, 2935, 0, 16, 43392, 43408),
    ("fscod", 2, 0, 32, 34, 43424, 43426),
    ("frmsizecod", 6, 36, 34, 40, 43426, 43432),
    ("bsid", 5, 6, 40, 45, 43432, 43437),
    ("bsmod", 3, 0, 45, 48, 43437, 43440),
    ("acmod", 3, 7, 48, 51, 43440, 43443),
    ("cmixlev", 2, 0, 51, 53, 43443, 43445),
    ("surmixlev", 2, 0, 53, 55, 43445, 43447),
    ("lfeon", 1, 1, 55, 56, 43447, 43448),
    ("dialnorm", 5, 31, 56, 61, 43448, 43453),
    ("compre", 1, 1, 61, 62, 43453, 43454),
    ("compr", 8, 1, 62, 70, 43454, 43462),
    ("langcode", 1, 0, 70, 71, 43462, 43463),
    ("audprodie", 1, 0, 71, 72, 43463, 43464),
    ("copyrightb", 1, 1, 72, 73, 43464, 43465),
    ("origbs", 1, 1, 73, 74, 43465, 43466),
    ("xbsi1e", 1, 1, 74, 75, 43466, 43467),
    ("dmixmod", 2, 1, 75, 77, 43467, 43469),
    ("ltrtcmixlev", 3, 4, 77, 80, 43469, 43472),
    ("ltrtsurmixlev", 3, 4, 80, 83, 43472, 43475),
    ("lorocmixlev", 3, 4, 83, 86, 43475, 43478),
    ("lorosurmixlev", 3, 4, 86, 89, 43478, 43481),
    ("xbsi2e", 1, 1, 89, 90, 43481, 43482),
    ("dsurexmod", 2, 1, 90, 92, 43482, 43484),
    ("dheadphonmod", 2, 0, 92, 94, 43484, 43486),
    ("adconvtyp", 1, 0, 94, 95, 43486, 43487),
    ("xbsi2", 8, 0, 95, 103, 43487, 43495),
    ("encinfo", 1, 0, 103, 104, 43495, 43496),
    ("addbsie", 1, 0, 104, 105, 43496, 43497),
]


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(f"FAIL {message}")


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


manifest_bytes = (ROOT / "MANIFEST.json").read_bytes()
manifest_hash = sha256(manifest_bytes)
manifest = json.loads(manifest_bytes)
require(manifest.get("package_id") == PACKAGE_ID, "package id")
require(
    (ROOT / "MANIFEST.sha256").read_text(encoding="ascii").split()[0] == manifest_hash,
    "manifest digest",
)
files = manifest.get("files")
expected_names = {"CHECK.py", "FIELD-VALUES.json", "REPORT.md", "SOURCE-RECEIPTS.json"}
require(isinstance(files, list) and {item.get("path") for item in files} == expected_names, "exact payload inventory")
for item in files:
    path = ROOT / item["path"]
    data = path.read_bytes()
    require(len(data) == item["bytes"] and sha256(data) == item["sha256"], f"payload hash {item['path']}")
seal = json.loads((ROOT / "package_seal.json").read_text(encoding="utf-8"))
require(seal.get("manifest_sha256") == manifest_hash and seal.get("files") == files, "package seal")
values = json.loads((ROOT / "FIELD-VALUES.json").read_text(encoding="utf-8"))
require(values["source_sha256"] == SOURCE_SHA256, "source SHA")
fields = values["bsi_fields"]
require(len(fields) == len(EXPECTED), "field count")
for item, expected in zip(fields, EXPECTED):
    name, width, value, frame_start, frame_end, file_start, file_end = expected
    require(item["name"] == name and item["present"] is True, f"field presence/order {name}")
    require(item["width_bits"] == width and item["value"] == value, f"field value {name}")
    require(item["frame_bits"] == [frame_start, frame_end], f"frame offset {name}")
    require(item["file_bits"] == [file_start, file_end], f"file offset {name}")
require(values["addbsi_payload"] == {"present": False, "bytes": 0}, "addbsi absent")
require(values["target_transfer"]["user_target_rates_kbps"] == {"i0": 640, "d0": 1024, "aggregate": 1664}, "target rates")
require(values["target_transfer"]["target_output_observed"] is False, "sample/target distinction")
require(values["remaining_gates"]["d0_audio_algorithm"] == "UNBOUND", "D0 audio gate")
require(values["remaining_gates"]["complete_1664_kbps_output"] == "NOT_RUN", "target output status")
source = json.loads((ROOT / "SOURCE-RECEIPTS.json").read_text(encoding="utf-8"))["source"]
require(source["sha256"] == SOURCE_SHA256 and source["source_path_included"] is False, "sanitized source identity")
require(source["raw_media_or_frame_bytes_included"] is False, "raw media exclusion")
print(f"PASS package={PACKAGE_ID} manifest={manifest_hash}")
print("PASS full observed I0 BSI through addbsie; addbsi absent; target remains unobserved")
