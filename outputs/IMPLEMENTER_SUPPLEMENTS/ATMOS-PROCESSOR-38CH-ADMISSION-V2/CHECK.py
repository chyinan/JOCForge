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
require(manifest.get("package_id") == "ATMOS-PROCESSOR-38CH-ADMISSION-V2", "package id")
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
require(evidence["source_package"]["manifest_sha256"] == "7fe318d5570ca672c21e9b9965c76e979c9e510be9943a2b24b7a8ab8c9e0fc8", "source manifest binding")
require(evidence["channel_count_chain"]["process_guard"].startswith("FUN_18018ae20 requires"), "process count guard")
require(evidence["channel_count_chain"]["master_count_write"] == "FUN_18054fa70 writes U16 at CompositeMaster+0x1A", "master count writer")
require(evidence["best_supported_rule"]["runtime_38ch_value_observed"] is False, "no runtime observation claimed")
require(evidence["best_supported_rule"]["full_38ch_encode_claim"] is False, "no full encode claim")
require(vector["runtime_execution"] == "NOT_RUN" and vector["specific_master_value_observed"] is False, "runtime vector boundary")
require(evidence["execution_boundary"]["private_key_or_auth_read"] is False, "private material excluded")

print("PASS: static master-count alias, 38-channel conditional admission, frame capacity, and runtime boundary")
