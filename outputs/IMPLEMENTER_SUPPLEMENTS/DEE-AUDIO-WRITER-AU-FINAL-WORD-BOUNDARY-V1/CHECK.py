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
require(manifest.get("package_id") == "DEE-AUDIO-WRITER-AU-FINAL-WORD-BOUNDARY-V1", "package id")
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
require(evidence["source_package"]["manifest_sha256"] == "2c4c61f399d3139ccf07e7fa781b5c59e94fe33f748f0e8412ec7b653888a9d7", "source manifest binding")
require(evidence["same_writer_state"]["same_instance_confidence"] == "HIGH", "writer state join")
require(evidence["field_close_and_au_unit_flush"]["span_close_flushes_accumulator"] is False, "field close is not flush")
require(evidence["field_close_and_au_unit_flush"]["flush_condition"].startswith("after successful FUN_180093360"), "flush condition")
require(evidence["field_close_and_au_unit_flush"]["flush_advances_pointer_or_clears_cursor"] is False, "flush state effect")
require(evidence["downstream_open_edge"]["alias_to_uVar28_writer_pointer"] == "UNBOUND", "downstream alias remains open")
require(vector["execution"] == "NOT_RUN" and vector["product_au_or_audio"] is False, "no product validation claim")
require(evidence["exclusions"]["private_key_or_license_material_included"] is False, "private material exclusion")

print("PASS: residual U16 final-word contract, per-unit reset, and downstream AU boundary")
