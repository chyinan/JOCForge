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
require(manifest.get("package_id") == "F06-R138-MEDIAMETADATA-ROW-WRITER-MVP-V2", "package id")
require(manifest.get("source_reverse_manifest_sha256") == "49fac997e42b61aa2a98af32c7698e2f3baf129b6d4ae96138dc96e4f7fcc8bd", "source package")

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
facts = evidence.get("host_and_type_facts", {})
require(facts.get("host_method") == "OutputWrapper::SetProperty", "host method")
require(facts.get("selected_process_state_member") == "+0xA0", "process-state field")
require(facts.get("generic_vcall_target_bound") is False, "generic dispatch boundary")
require(evidence.get("row_transform", {}).get("output_index_rule") == "one_based index = zero_based input ordinal + 1", "row index rule")
require(evidence.get("converter_typed_crosswalk", {}).get("same_ordinal_typed_field_comparison") is True, "typed crosswalk")
require(vector.get("expected_output_included") is False and vector.get("executed") is False, "input-only vector")
require(sources.get("private_key_or_license_material_copied") is False, "private material boundary")

print("PASS: manifest/payload hashes, host candidate, row transform, typed ADM crosswalk, and exclusions")
