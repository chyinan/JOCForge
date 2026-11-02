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
require(sha256(manifest_path) == (ROOT / "MANIFEST.sha256").read_text(encoding="ascii").split()[0], "manifest hash")
require(manifest.get("package_id") == "ATMOS-JOC-INNER-PREPROCESSOR-DISPATCH-V2", "package id")
payload = manifest.get("files")
require(isinstance(payload, list) and len(payload) == 4, "payload inventory")
actual_names = {path.name for path in ROOT.iterdir() if path.is_file() and path.name not in {"MANIFEST.json", "MANIFEST.sha256", "package_seal.json"}}
require(actual_names == {entry.get("file") for entry in payload}, "payload file set")
for entry in payload:
    path = ROOT / entry["file"]
    require(path.stat().st_size == entry["bytes"], f"size {entry['file']}")
    require(sha256(path) == entry["sha256"], f"hash {entry['file']}")

seal = json.loads((ROOT / "package_seal.json").read_text(encoding="utf-8"))
sealed = seal.get("files")
require(isinstance(sealed, list) and seal.get("file_count") == len(sealed), "seal inventory")
require({entry.get("file") for entry in sealed} == actual_names | {"MANIFEST.json", "MANIFEST.sha256"}, "sealed file set")
for entry in sealed:
    path = ROOT / entry["file"]
    require(path.stat().st_size == entry["bytes"], f"sealed size {entry['file']}")
    require(sha256(path) == entry["sha256"], f"sealed hash {entry['file']}")

evidence = json.loads((ROOT / "EVIDENCE.json").read_text(encoding="utf-8"))
vector = json.loads((ROOT / "INPUT-ONLY-VECTOR.json").read_text(encoding="utf-8"))
require(evidence["source_package"]["manifest_sha256"] == "6445cd3a5ef2bf9719fb2714c02d5dc9e6e49da1f9314a304ba8d67fdb1b8b21", "source manifest binding")
require(evidence["source_package"]["package_seal_sha256"] == "7d4b92b114ff196a73fd926c692968159e229c592ecd8a159450ca8d8aeadb71", "source seal binding")
require(evidence["selector_zero"]["plus_0x28"]["classification"].startswith("state/control update"), "zero branch +0x28 class")
require(evidence["selector_zero"]["sample_process"]["slot"] == "+0x08", "spatial sample process slot")
require(evidence["selector_nonzero"]["actual_indirect_target"] == "UNBOUND/NO", "bypass target stays unbound")
require(evidence["sample_scope"]["selector_recorded_in_receipts"] is False, "per-run selector remains unbound")
require(vector["execution"] == "NOT_RUN" and vector["selector_default_inferred"] is False, "input-only vector")
require(evidence["exclusions"]["private_key_or_license_material_included"] is False, "private material exclusion")

print("PASS: selector-conditional inner dispatch, separate sc_process slot, and unresolved per-run selector boundary")
