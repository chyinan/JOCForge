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
require(manifest.get("package_id") == "F13C-DMIXMOD-NO-PROMPT-INPUT-CONTRACT-V1", "package id")
require(manifest.get("source_reverse_manifest_sha256") == "f0c42ba60d2de6105e2662c10b14dc4b22cdd4cd9b9069068010233159916cfa", "source package manifest")

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
require(evidence.get("input_contract", {}).get("user_prompt_required") is False, "no prompt rule")
require(evidence.get("input_contract", {}).get("selected_attempt10_template_preference") == "loro", "selected template preference")
require(evidence.get("input_contract", {}).get("selected_attempt10_template_levels_db") == [-3,-3,-3,-3], "selected template levels")
require(evidence.get("native_static_boundary", {}).get("loro_to_native_ddp_dmixmod") == "UNBOUND", "native mapping boundary")
require(evidence.get("native_static_boundary", {}).get("native_dsp_consumer_found") is False, "no unsupported DSP claim")
require(vector.get("expected_runtime_frame_bits_included") is False and vector.get("executed") is False, "input-only boundary")
require(sources.get("implementation_source_hashes_recomputed_in_wash") is True, "current source hash check")
require(sources.get("private_key_or_license_material_copied") is False, "private material exclusion")

print("PASS: manifest/payload hashes, no-prompt contract, source template values, native-effect boundary, and exclusions")
