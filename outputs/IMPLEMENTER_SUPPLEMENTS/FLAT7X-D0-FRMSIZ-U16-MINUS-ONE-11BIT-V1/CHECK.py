from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent


def read_json(name: str) -> dict:
    return json.loads((ROOT / name).read_text(encoding="utf-8"))


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


manifest = read_json("MANIFEST.json")
seal = read_json("package_seal.json")
evidence = read_json("EVIDENCE.json")
source = read_json("SOURCE-PACKAGE.json")

payloads = manifest["payloads"]
payload_names = {entry["file"] for entry in payloads}
actual_names = {
    path.name
    for path in ROOT.iterdir()
    if path.is_file() and path.name not in {"MANIFEST.json", "package_seal.json"}
}
if payload_names != actual_names:
    raise SystemExit("payload inventory mismatch")

for entry in payloads:
    path = ROOT / entry["file"]
    if path.stat().st_size != entry["bytes"] or digest(path) != entry["sha256"]:
        raise SystemExit(f"payload integrity mismatch: {entry['file']}")

if seal["manifest_sha256"] != digest(ROOT / "MANIFEST.json"):
    raise SystemExit("manifest hash mismatch")
if seal["payload_hashes"] != payloads:
    raise SystemExit("seal payload rows mismatch")
if manifest["implementation_ready"] is not False or manifest["production_allowed"] is not False:
    raise SystemExit("production disposition mismatch")
if evidence["target_contract"]["d0_frame_bytes"] // 2 - 1 != evidence["target_contract"]["expected_frmsiz_code"]:
    raise SystemExit("target frmsiz formula mismatch")
if evidence["native_static_evidence"]["writer_width_bits"] != 11:
    raise SystemExit("writer width mismatch")
if evidence["native_static_evidence"]["target_d0_state_join"] != "UNBOUND":
    raise SystemExit("owner boundary mismatch")
if source["source_manifest_sha256"] != evidence["native_static_evidence"]["source_manifest_sha256"]:
    raise SystemExit("source manifest binding mismatch")

print("PASS: Flat7x D0 frmsiz contract, source hashes, 11-bit writer, and unbound owner disposition")
