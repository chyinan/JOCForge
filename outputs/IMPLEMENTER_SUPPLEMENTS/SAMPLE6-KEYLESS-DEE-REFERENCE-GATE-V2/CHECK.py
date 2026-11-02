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
require(manifest.get("package_id") == "SAMPLE6-KEYLESS-DEE-REFERENCE-GATE-V2", "package id")
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
sources = json.loads((ROOT / "SOURCE-IDENTITIES.json").read_text(encoding="utf-8"))
require(evidence["source_package"]["manifest_sha256"] == "3070124494fcbfdf8d5da3e0578535005266c512892ce2b39b1bf198891c8722", "source package binding")
require(evidence["dee_reference"]["status"] == "NOT_RUN" if "dee_reference" in evidence else evidence["audio_difference_status"]["fresh_dee_output"] == "NOT_RUN", "fresh DEE run remains NOT_RUN")
require(evidence["audio_difference_status"]["audio_equivalence"] == "NOT_ESTABLISHED", "no equivalence claim")
require(evidence["constraints"]["dee_invoked"] is False, "DEE not invoked")
require(evidence["constraints"]["license_or_private_key_read"] is False, "key untouched")
require(sources["authorized_input_sha256"] == evidence["current_jocforge_output"]["input_sha256"], "input identity")
require(sources["current_jocforge_output_sha256"] == evidence["current_jocforge_output"]["output_sha256"], "output identity")
require(evidence["current_jocforge_output"]["ffprobe_whole_stream_parse"] == "PASS", "parse result")
require(evidence["current_jocforge_output"]["ffmpeg_decode_only_exit_code"] == 0, "decode result")
require(evidence["current_jocforge_output"]["pcm_persisted"] is False, "PCM not persisted")

print("PASS: current output parse/decode, private-key boundary, and fresh DEE reference NOT_RUN")
