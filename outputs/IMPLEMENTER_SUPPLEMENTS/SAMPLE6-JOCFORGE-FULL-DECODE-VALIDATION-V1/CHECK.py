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
require(manifest.get("package_id") == "SAMPLE6-JOCFORGE-FULL-DECODE-VALIDATION-V1", "package id")
payload = manifest.get("files")
require(isinstance(payload, list) and len(payload) == 3, "payload inventory")
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
require(evidence["input"]["sha256"] == "7826dcbef37ef69642a6164c97a37bcfb3f917cea74aa0f0a30d76d0a825f1d1", "input identity")
require(evidence["run"]["exit_code"] == 0 and evidence["output"]["published_access_units"] == 4247, "run completion")
require(evidence["output"]["sha256"] == "d9680298ac1d913f408b77a72b45171fc379dabb71de80e44c24d27cbf1de6da", "output identity")
require(evidence["independent_validation"]["whole_stream_parse"] == "PASS", "parser result")
require(evidence["independent_validation"]["full_stream_decode_exit_code"] == 0, "decoder result")
require(evidence["audio_affecting_fields"]["output_u5"]["value"] == 17, "U5 output observation")
require(evidence["audio_affecting_fields"]["same_input_dee_reference"]["u5"] == 21, "same-input DEE receipt")
require(evidence["audio_affecting_fields"]["fresh_same_input_dee_bsi_comparison"] == "NOT_RUN", "fresh BSI comparison remains not run")
require(evidence["comparison_boundary"]["audio_essence_equivalence"] == "NOT_ESTABLISHED", "no audio equivalence claim")
require(evidence["exclusions"]["pcm_saved"] is False and evidence["exclusions"]["private_key_or_license_material_read"] is False, "payload/key exclusion")

print("PASS: full-track parse/decode receipt, same-input U5 mismatch, BSI comparison NOT_RUN, and payload exclusion")
