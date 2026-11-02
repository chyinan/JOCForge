from __future__ import annotations

import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent


def require(condition: bool, label: str) -> None:
    if not condition:
        raise SystemExit(f"FAIL {label}")


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


manifest_path = ROOT / "MANIFEST.json"
manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
sidecar = (ROOT / "MANIFEST.sha256").read_text(encoding="ascii").split()[0]
require(sha256(manifest_path) == sidecar, "manifest sidecar")
require(manifest.get("package_id") == "EAC3-EXPONENT-U4-U7-WRITER-BOUNDARY-V1", "package id")
require(manifest.get("source_reverse_manifest_sha256") == "60dd1b42814edd97c44ad827c2a06b7cd1fd9f39b3ca4303422f6de5650898cf", "source manifest hash")
require(manifest.get("implementation_trial_ready") is True, "trial disposition")

files = manifest.get("files")
require(isinstance(files, list) and bool(files), "file inventory")
names = {entry.get("file") for entry in files}
actual = {path.name for path in ROOT.iterdir() if path.is_file() and path.name not in {"MANIFEST.json", "MANIFEST.sha256"}}
require(names == actual, "sealed file inventory")
for entry in files:
    path = ROOT / entry["file"]
    require(path.is_file(), f"missing {entry['file']}")
    require(path.stat().st_size == entry["bytes"], f"size {entry['file']}")
    require(sha256(path) == entry["sha256"], f"hash {entry['file']}")

evidence = json.loads((ROOT / "EVIDENCE.json").read_text(encoding="utf-8"))
vectors = json.loads((ROOT / "INPUT-ONLY-VECTORS.json").read_text(encoding="utf-8"))
sources = json.loads((ROOT / "SOURCE-MANIFESTS.json").read_text(encoding="utf-8"))
fields = evidence.get("exponent_fields", {})
require(fields.get("u4", {}).get("width_bits") == 4, "U4 width")
require(fields.get("u7", {}).get("width_bits") == 7, "U7 width")
require(fields.get("u7", {}).get("count_field") == "u16(state+0x141c8+2*j)", "group count field")
require(fields.get("shared_cursor", {}).get("advance_bits") == "4+7*N", "shared cursor")
require(evidence.get("parser_error_relation", {}).get("sole_root_cause_established") is False, "root cause boundary")
require(vectors.get("native_dee_observation") is False, "input-only status")
require(evidence.get("confidence_and_unbound", {}).get("real_adm_frame_group_count") == "UNBOUND", "runtime count boundary")
require(sources.get("binary_copied") is False and sources.get("private_key_or_license_material_copied") is False, "exclusions")

print("PASS: manifest and payload hashes, exact U4/U7 widths/count/cursor, input-only vectors, and exclusions")
