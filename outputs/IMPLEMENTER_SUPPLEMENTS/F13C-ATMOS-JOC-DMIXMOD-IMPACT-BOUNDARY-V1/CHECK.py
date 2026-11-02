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
require(manifest.get("package_id") == "F13C-ATMOS-JOC-DMIXMOD-IMPACT-BOUNDARY-V1", "package id")
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
require(evidence["source_package"]["manifest_sha256"] == "b0c7ef16fc3bbe6767039142fb8c0666a1de48a21d3a5b5428ddb6fb0097dabd", "source manifest binding")
require(evidence["source_package"]["sha256sum_files_verified"] == 16, "source file checks")
require(evidence["native_path_boundary"]["first_reached_consumer"] == "FUN_1404807d0 generic name/value/error result-tree builder", "first consumer")
require(evidence["native_path_boundary"]["audio_dsp_or_postmix_consumer_in_scoped_path"] == "NOT_OBSERVED", "DSP effect boundary")
require(evidence["best_supported_rule"]["mvp_blocked"] is False, "MVP gate")
require(vector["encoder_execution"] == "NOT_RUN" and vector["native_audio_effect"] == "UNBOUND", "runtime/effect boundary")
require(evidence["execution_boundary"]["private_key_or_license_material_read"] is False, "private material exclusion")

print("PASS: selected Atmos route source values, generic validator consumer boundary, and no-prompt rule")
