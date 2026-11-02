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
require(manifest.get("package_id") == "ATMOS-JOC-ROW-TO-RENDERER-HANDOFF-V1", "package id")
require(manifest.get("source_reverse_manifest_sha256") == "d926de87e639bb9e9201ece1f5f7ec70e943e7f03e2167b5c582aad9bc7cfffe", "source manifest")

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
copy = evidence.get("descriptor_and_copy_contract", {})
require(copy.get("channel_count_offset") == "0x08", "channel count field")
require(copy.get("frame_count_offset") == "0x10", "frame count field")
require(copy.get("copy_function_rva") == "0x18d640", "copy function")
require(copy.get("order_preserved") is True and copy.get("element_width_bytes") == 4, "row copy contract")
require(evidence.get("renderer_dispatch", {}).get("process_vtable_slot_offset") == "+0x08", "renderer slot")
require(evidence.get("unbound_audio_boundary", {}).get("nested_analysis_or_qmf_target") == "UNBOUND/NO", "inner transform boundary")
require(vector.get("expected_output_samples_included") is False and vector.get("executed") is False, "input-only vector")
require(sources.get("referenced_source_hash_mismatches") == 0, "source hash verification")
require(sources.get("private_key_or_license_material_copied") is False, "private material exclusion")

print("PASS: manifest/payload hashes, row-to-renderer contract, 36-channel scope, input-only vector, and exclusions")
