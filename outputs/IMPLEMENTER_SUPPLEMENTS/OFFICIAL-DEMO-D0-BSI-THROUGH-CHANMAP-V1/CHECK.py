"""Functional Core: validate one sanitized official-demo D0 BSI vector."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
EXPECTED_FILES = {"CHECK.py", "FIELD-VALUES.json", "REPORT.md", "SOURCE-RECEIPTS.json"}
EXPECTED_SOURCE = "f976aba7a4750be80b4373a5d7fabf2ad101e0c227fc4bd77716b76879d546cd"
EXPECTED_FIELDS = [
    ("syncword", 16, 2935, 0, 16),
    ("strmtyp", 2, 1, 16, 18),
    ("substreamid", 3, 0, 18, 21),
    ("frmsiz", 11, 1279, 21, 32),
    ("fscod", 2, 0, 32, 34),
    ("numblkscod", 2, 3, 34, 36),
    ("acmod", 3, 5, 36, 39),
    ("lfeon", 1, 0, 39, 40),
    ("bsid", 5, 16, 40, 45),
    ("dialnorm", 5, 31, 45, 50),
    ("compre", 1, 1, 50, 51),
    ("compr", 8, 1, 51, 59),
    ("chanmape", 1, 1, 59, 60),
    ("chanmap", 16, 6656, 60, 76),
]


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(f"CHECK=FAIL {message}")


def main() -> int:
    manifest_path = ROOT / "manifest.json"
    seal_path = ROOT / "package_seal.json"
    fields_path = ROOT / "FIELD-VALUES.json"
    source_path = ROOT / "SOURCE-RECEIPTS.json"

    manifest_bytes = manifest_path.read_bytes()
    manifest_hash = hashlib.sha256(manifest_bytes).hexdigest()
    manifest = json.loads(manifest_bytes)
    require(manifest["package_id"] == "OFFICIAL-DEMO-D0-BSI-THROUGH-CHANMAP-V1", "package id")
    require(manifest["status"] == "WASH_APPROVED_SAMPLE_D0_BSI_THROUGH_CHANMAP; PRODUCT_AUDIO_NOT_READY", "status")

    entries = manifest["files"]
    require({entry["path"] for entry in entries} == EXPECTED_FILES, "payload inventory")
    for entry in entries:
        payload = ROOT / entry["path"]
        require(payload.is_file(), f"missing {entry['path']}")
        require(payload.stat().st_size == entry["bytes"], f"size {entry['path']}")
        require(sha256(payload) == entry["sha256"], f"hash {entry['path']}")

    manifest_sha = (ROOT / "MANIFEST.sha256").read_text(encoding="ascii").split()[0]
    require(manifest_sha == manifest_hash, "detached manifest hash")
    seal = json.loads(seal_path.read_text(encoding="utf-8"))
    require(seal["manifest_sha256"] == manifest_hash, "seal manifest hash")
    require(seal["files"] == entries, "seal inventory")

    field_document = json.loads(fields_path.read_text(encoding="utf-8"))
    source_receipts = json.loads(source_path.read_text(encoding="utf-8"))
    require(field_document["source_sha256"] == EXPECTED_SOURCE, "field source identity")
    require(source_receipts["source"]["sha256"] == EXPECTED_SOURCE, "source receipt identity")
    require(source_receipts["source"]["raw_media_or_frame_bytes_in_package"] is False, "raw media exclusion")
    require(source_receipts["source"]["parser_source_in_package"] is False, "parser source exclusion")
    require(field_document["reference_access_unit"]["frame_order"] == ["I0", "D0"], "sample frame order")

    d0 = field_document["d0_sample_frame"]
    require(d0["file_byte_offset"] == 7984 and d0["frame_bytes"] == 2560, "sample D0 extent")
    require(d0["observed_frmsiz"] == 1279 and d0["sample_rate_kbps"] == 640, "sample D0 rate fields")
    rows = d0["fields_through_chanmap"]
    require(len(rows) == len(EXPECTED_FIELDS), "field count")
    for row, (name, width, value, start, end) in zip(rows, EXPECTED_FIELDS, strict=True):
        require((row["name"], row["width_bits"], row["value"], row["frame_bit_start"], row["frame_bit_end"]) == (name, width, value, start, end), f"field {name}")
        require(row["file_bit_start"] == 63872 + start and row["file_bit_end"] == 63872 + end, f"file bits {name}")
    require(rows[-1]["hex_value"] == "0x1A00", "numeric chanmap")
    require(d0["stop_after"] == "chanmap" and d0["next_field_read"] is False, "read boundary")

    target = field_document["target_rate_contract"]
    require((target["target_rate_kbps"], target["target_frame_bytes"], target["target_frmsiz"]) == (1024, 4096, 2047), "separate target rate contract")
    require(target["evidence_kind"] == "SEPARATE_TARGET_CONTRACT_NOT_SAMPLE_OBSERVATION", "target evidence tier")
    print(f"PACKAGE_CHECK=PASS package={manifest['package_id']} fields={len(rows)} chanmap=0x1A00 sample_d0=640kbps target_d0=1024kbps")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
