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
require(manifest.get("package_id") == "EAC3-EXPONENT-CODE-PRODUCER-U4-U7-V1", "package id")
require(manifest.get("source_reverse_manifest_sha256") == "ad95d8576957c0ed20d8339c322c01c9669ab48ea8bd545e28b3ab7400c32ec6", "source package")

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
join = evidence.get("typed_callsite_join", {})
require(join.get("producer", {}).get("pointer_base") == join.get("writer", {}).get("pointer_base") == "state+0x140f0", "typed producer/writer base")
fields = evidence.get("fullband_lane_fields", {})
require(fields.get("group_count_N") == "i16(state+0x141c8+2*j)", "N field")
require(fields.get("difference_count_D") == "i16(state+0x1418c+10*block+2*lane)", "D field")
require(evidence.get("writer_rule", {}).get("advance_bits") == "4+7*N", "cursor bits")
require(evidence.get("remaining_boundaries", {}).get("upstream_N_assignment") == "UNBOUND", "N producer boundary")
require(vectors.get("native_dee_observation") is False and vectors.get("expected_output_bytes_included") is False, "synthetic vector boundary")
require(evidence.get("payload_policy", {}).get("private_key_or_license_material_included") is False, "private material boundary")

print("PASS: manifest/payload hashes, typed U4/U7 producer-writer join, input-only vectors, and exclusions")
