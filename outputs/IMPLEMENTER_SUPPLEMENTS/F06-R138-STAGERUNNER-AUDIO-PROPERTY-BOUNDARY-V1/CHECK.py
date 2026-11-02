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
require(manifest.get("package_id") == "F06-R138-STAGERUNNER-AUDIO-PROPERTY-BOUNDARY-V1", "package id")
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
require(evidence["source_package"]["manifest_sha256"] == "986e4138a39e83e35a3f0622a743206f5b1288d4b2d290e2947ac083aa8e9cf9", "source manifest binding")
require(evidence["bound_callsites"]["initoutput_audio_property"] == "dee_audio_input_source from StageRunner+0x428", "audio input property")
require(evidence["bound_callsites"]["initoutput_video_property"] == "dee_video_input_source from StageRunner+0x5E8", "video input property")
require(evidence["bound_callsites"]["dee_audio_source_type_in_this_window"] is False, "source type not in window")
require(evidence["bound_callsites"]["writes_outputwrapper_plus_0x178"] is False, "no registry write claimed")
require(vector["runtime_execution"] == "NOT_RUN", "no runtime claim")
require(evidence["execution_boundary"]["private_key_or_license_material_included"] is False, "private material exclusion")

print("PASS: two InitOutput property calls distinguished from the unresolved source-registry producer")
