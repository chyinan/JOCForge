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
require(manifest.get("package_id") == "EC3-AU0-EXPONENT-PARSER-FAILURE-V1", "package id")
require(manifest.get("source_reverse_manifest_sha256") == "fdf37f4459b3572f2925dff3e8f95f9f03d95be53cb839ed0ca8b0b89a93fb73", "incoming manifest")

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
parser = evidence.get("parser_observation", {})
require(parser.get("demux_format") == "eac3" and parser.get("first_access_unit") == 0, "AU0 parser observation")
require(parser.get("first_decoder_error", "").startswith("exponent -1 is out-of-range"), "decoder error")
require(parser.get("first_packet_size_bytes") == 4096 and parser.get("pcm_saved") is False, "packet/PCM boundary")
require(evidence.get("artifact_identity", {}).get("adm_input_identity") == "NOT_BOUND_IN_SOURCE_PACKAGE", "input-identity limit")
require(evidence.get("scope_exclusions", {}).get("private_key_or_license_read") is False, "private material boundary")

print("PASS: manifest/payload hashes, AU0 E-AC-3 exponent failure, artifact scope, and exclusions")
