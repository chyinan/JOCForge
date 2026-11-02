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
require(manifest.get("package_id") == "F06-R138-SETTER-RECEIVER-VTABLE-V2", "package id")
require(manifest.get("source_reverse_manifest_sha256") == "40c4a413f09b31b4b21a5d1fbc992128f8e1396d68ab0c271c1f38ed50b53208", "source manifest")

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
require(evidence.get("host_call", {}).get("outer_receiver_class") == "dee::OutputWrapper", "outer receiver")
require(evidence.get("host_call", {}).get("argument_dynamic_type") == "dee::Types::StringType", "argument type")
require(evidence.get("target_candidate", {}).get("plvar3_vtable_type") == "UNBOUND", "nested receiver boundary")
require(evidence.get("target_candidate", {}).get("generic_vcall_target") == "UNBOUND", "target boundary")
require(evidence.get("target_candidate", {}).get("candidate_method") == "dee::Types::MediaMetadata::set", "candidate method")
require(evidence.get("scope", {}).get("key_or_license_read") is False, "secret boundary")

print("PASS: manifest/payload hashes, outer/inner receiver distinction, candidate slot, and exclusions")
