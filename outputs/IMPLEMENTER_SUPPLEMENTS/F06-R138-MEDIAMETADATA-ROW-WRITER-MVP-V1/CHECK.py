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
require(manifest.get("package_id") == "F06-R138-MEDIAMETADATA-ROW-WRITER-MVP-V1", "package id")
require(manifest.get("source_reverse_manifest_sha256") == "3a3bf13ff538a7f5c90e8700ec2a0b9d31d16c31182716b3b5813d42602420ad", "source manifest")

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
vector = json.loads((ROOT / "INPUT-ONLY-VECTOR.json").read_text(encoding="utf-8"))
sources = json.loads((ROOT / "SOURCE-MANIFESTS.json").read_text(encoding="utf-8"))
rule = evidence.get("media_metadata_type", {})
require(rule.get("safe_payload_type") == "vector<map<string,string>>", "typed payload")
require(rule.get("metadata_index_formula") == "one_based_index = zero_based_source_row_ordinal + 1", "row index rule")
require(rule.get("input_order_preserved") is True and rule.get("filename_copied") is True, "row copy rule")
crosswalk = evidence.get("converter_typed_crosswalk", {})
require(crosswalk.get("converter_internal_crosswalk") == "BOUND_STATIC_TYPED_FIELD_COMPARISONS", "converter typed crosswalk")
require(evidence.get("unbound_or_not_run", {}).get("selected_r138_setter_invocation") == "UNBOUND", "selected setter limit")
require(vector.get("expected_output_rows_included") is False and vector.get("executed") is False, "input-only vector")
require(sources.get("private_key_or_license_material_copied") is False, "private material boundary")

print("PASS: manifest/payload hashes, typed row-writer rule, ADM ID crosswalk, scoped UNBOUND, and exclusions")
