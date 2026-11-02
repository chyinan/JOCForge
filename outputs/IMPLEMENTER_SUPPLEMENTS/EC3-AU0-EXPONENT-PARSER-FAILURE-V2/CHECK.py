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
require(manifest.get("package_id") == "EC3-AU0-EXPONENT-PARSER-FAILURE-V2", "package id")
require(manifest.get("source_reverse_manifest_sha256") == "fa55517ec872c0160b95c8ff66b75f47e5ad8e408dc92a8cdb1c2d6e39a4a52e", "source package")

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
parser = evidence.get("first_packet_observation", {})
require(parser.get("syncword_u16_big_endian") == "0x0b77" and parser.get("syncword_byte_offset") == 0, "syncword")
require(parser.get("first_access_unit") == 0 and parser.get("packet_size_bytes") == 4096, "first AU")
require("exponent -1 is out-of-range" in parser.get("ffprobe_first_error", ""), "AU0 error")
require(parser.get("raw_bytes_saved_or_included") is False and parser.get("ffmpeg_pcm_saved") is False, "payload boundary")
require(evidence.get("dee_writer_context", {}).get("bit_cursor_before_first_u4_for_rust_artifact") == "UNBOUND", "cursor boundary")
require(evidence.get("scope_exclusions", {}).get("private_key_or_license_read") is False, "private material boundary")

print("PASS: manifest/payload hashes, syncword, AU0 exponent failure, U4/U7 cursor scope, and exclusions")
