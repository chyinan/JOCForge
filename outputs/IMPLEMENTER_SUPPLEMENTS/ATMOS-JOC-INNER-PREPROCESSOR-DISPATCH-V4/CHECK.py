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
require(manifest.get("package_id") == "ATMOS-JOC-INNER-PREPROCESSOR-DISPATCH-V4", "package id")
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
require(evidence["source_package"]["manifest_sha256"] == "bcd366ee63916707421278c34ba430b8b35bede053e65f6f8de13963b35ce8bc", "source manifest binding")
require(evidence["source_package"]["package_seal_sha256"] == "960fafcb65249bb846a7dbd46e7dd0aad966a04b848f9078cdf4fb361ee94645", "source seal binding")
require(evidence["selected_id0_lookup"]["lookup_type_id"] == 0, "selected ID")
require(evidence["selected_id0_lookup"]["selector_getter"] == "selected_object.vtable+0x38 in FUN_180435120", "selector field contract")
require(evidence["base_getter_candidate"]["returned_field"] == "object+0x58", "base getter field")
require("no literal default" in evidence["base_getter_candidate"]["field_source"], "no DLL constant default")
require(evidence["rerenderbus_config_candidate"]["selected_by_type_id_0_lookup"] is False, "Rerender candidate excluded")
require(evidence["best_supported_contract"]["per_run_selector_value"] == "UNBOUND/NOT_RECORDED", "per-run value remains unbound")
require(evidence["best_supported_contract"]["ui_or_prompt_requirement"] == "NOT_ESTABLISHED_BY_THIS_EVIDENCE", "no UI prompt inferred")
require(vector["execution"] == "NOT_RUN" and vector["default_inferred"] is False, "input-only limits")
require(evidence["exclusions"]["private_key_or_license_material_included"] is False, "private material exclusion")

print("PASS: ID-0 getter field source, caller-filled selector, Rerender exclusion, and no-default/no-prompt boundary")
