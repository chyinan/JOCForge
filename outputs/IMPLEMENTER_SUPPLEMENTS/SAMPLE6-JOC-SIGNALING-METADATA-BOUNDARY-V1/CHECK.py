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
require(manifest.get("package_id") == "SAMPLE6-JOC-SIGNALING-METADATA-BOUNDARY-V1", "package id")
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
require(evidence["source_package"]["manifest_sha256"] == "42194079cf43f558e251e53e545731c056e7fd1d3c284073fe2cb8ed5be992b1", "source manifest binding")
require(evidence["independently_bound_output"]["output_sha256"] == "d9680298ac1d913f408b77a72b45171fc379dabb71de80e44c24d27cbf1de6da", "output identity")
require(evidence["stream_metadata"]["profile"] == "Dolby Digital Plus + Dolby Atmos", "stream profile")
require(evidence["per_unit_metadata"]["packet_count"] == 4247, "packet count")
require(evidence["per_unit_metadata"]["explicit_per_au_joc_emdf_flag"] == "NOT_EXPOSED_BY_FFPROBE_METADATA", "per-AU flag boundary")
require(evidence["interpretation"]["claim_all_4247_aus_have_joc"] is False, "no full per-AU coverage claim")
require(evidence["access_boundary"]["payload_saved"] is False and evidence["access_boundary"]["pcm_saved"] is False, "payload/PCM boundary")

print("PASS: Atmos stream profile, 4,247-frame metadata scope, and no per-AU JOC claim")
