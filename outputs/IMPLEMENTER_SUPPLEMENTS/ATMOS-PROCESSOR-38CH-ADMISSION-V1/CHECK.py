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
require(manifest.get("package_id") == "ATMOS-PROCESSOR-38CH-ADMISSION-V1", "package id")
require(manifest.get("source_reverse_manifest_sha256") == "a5c7cfa973fccdcbbd38bd4a85ad5e3132acd585628257ca2bb34d315b0fe5ae", "source manifest")

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
vector = json.loads((ROOT / "INPUT-ONLY-VECTORS.json").read_text(encoding="utf-8"))
guard = evidence.get("channel_guard", {})
require(guard.get("guard_rule") == "descriptor.channel_count == configured_count", "channel guard")
require(guard.get("hard_coded_36_channel_maximum") is False, "no hard 36 limit")
require(evidence.get("frame_guard", {}).get("selected_profile_frame_samples") == 1536, "profile frame size")
require(evidence.get("hanmao_metadata", {}).get("runtime_descriptor_channel_count_observed") is False, "runtime join boundary")
require(vector.get("execution") == "NOT_RUN" and vector.get("expected_output_audio_included") is False, "input-only vector")
require(evidence.get("payload_policy", {}).get("private_key_or_license_material_included") is False, "private material boundary")

print("PASS: manifest/payload hashes, 38-channel guard condition, frame capacity, and exclusions")
