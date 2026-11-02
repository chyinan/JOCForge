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
require(manifest.get("package_id") == "F06-R138-SETTER-VCALL-CANDIDATE-V1", "package id")
require(manifest.get("source_reverse_manifest_sha256") == "7864cbefbc4eec08ca8da03707a998d14f9aae5323bda5935fa9bb21cf617f91", "source manifest")

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
call = evidence.get("host_call", {})
candidate = evidence.get("target_candidate", {})
require(call.get("argument_type") == "dee::Types::StringType", "argument type")
require(call.get("argument_object_is_receiver") is False, "argument/receiver split")
require(call.get("slot_offset") == "+0x28", "host slot")
require(candidate.get("generic_vcall_target") == "UNBOUND", "vcall target boundary")
require(candidate.get("most_likely_candidate") is True, "candidate disposition")
require(vector.get("expected_runtime_target_included") is False and vector.get("executed") is False, "input-only boundary")
require(evidence.get("payload_policy", {}).get("private_key_or_license_material_included") is False, "private material boundary")

print("PASS: manifest/payload hashes, StringType argument split, candidate slot, UNBOUND vcall, and exclusions")
