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
require(manifest.get("package_id") == "ATMOS-JOC-HOST-TO-PLUGIN-SELECTOR-BOUNDARY-V1", "package id")
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
require(evidence["source_package"]["manifest_sha256"] == "7c4cc1e54fcb81cbce5281d57934ce9b4daa2c5d8cbd1867752e164f3490089c", "source manifest binding")
require(evidence["source_package"]["package_seal_sha256"] == "244d8b98540b90ef1d471819c72ce73fc6d32a97820ca1f8a6830fbc62853b53", "source seal binding")
require(evidence["host_factory_edge"]["operation"] == "selected_config_object.vtable+0x30(this)", "host factory edge")
require(evidence["host_factory_edge"]["explicit_renderbus_vector_or_selector_argument"] is False, "host argument boundary")
require(evidence["atmos_plugin_edge"]["route"] == "encode_to_atmos_ddp", "selected route")
require(evidence["remaining_boundary"]["producer_or_adapter_supplying_renderbus_vector"] == "UNBOUND", "producer boundary")
require(evidence["best_supported_rule"]["hardcoded_default_recommended"] is False, "no default invented")
require(vector["execution"] == "NOT_RUN" and vector["pcm_or_payload_included"] is False, "input vector boundary")
require(evidence["exclusions"]["private_key_or_license_material_included"] is False, "private material exclusion")

print("PASS: host-to-Atmos factory edge, separate RenderBus selector input, and no-default boundary")
