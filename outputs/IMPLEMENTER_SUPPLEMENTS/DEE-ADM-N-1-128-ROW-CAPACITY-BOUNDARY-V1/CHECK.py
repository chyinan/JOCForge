from __future__ import annotations

import hashlib
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "DEE-ADM-N-1-128-ROW-CAPACITY-BOUNDARY-V1"
SOURCE_MANIFEST = "a3c5b0597dc5ab7d3b366ba87f05dccf55d543e65ba9cabbd8f35f639312f0a9"
SOURCE_SEAL = "cbe9dcbd3cb1a97e787718abc809f026f585d30d19ee383a697ef5dd135f29a0"
PAYLOADS = {"EVIDENCE.json", "INPUT-ONLY-VECTORS.json", "REPORT.md", "SOURCE-MANIFESTS.json"}


def require(value: bool, message: str) -> None:
    if not value:
        raise RuntimeError(f"FAIL {PACKAGE_ID}: {message}")


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


manifest_path = ROOT / "MANIFEST.json"
manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
seal = json.loads((ROOT / "package_seal.json").read_text(encoding="utf-8"))
evidence = json.loads((ROOT / "EVIDENCE.json").read_text(encoding="utf-8"))
require(manifest.get("package_id") == PACKAGE_ID, "package id")
require(manifest.get("source_manifest_sha256") == SOURCE_MANIFEST, "source manifest")
require(manifest.get("source_package_seal_sha256") == SOURCE_SEAL, "source seal")
require(seal.get("manifest_sha256") == sha256(manifest_path), "manifest hash")
require({row.get("name") for row in manifest.get("payloads", [])} == PAYLOADS | {"CHECK.py"}, "payload inventory")
for row in manifest["payloads"]:
    payload = ROOT / row["name"]
    require(payload.is_file() and payload.stat().st_size == row["bytes"], "payload size")
    require(sha256(payload) == row["sha256"], "payload hash")

gate = evidence["physical_n_gate"]
require(gate["static_accepted_range"] == {"minimum": 1, "maximum": 128}, "N gate range")
require(evidence["first_row_wrapper"]["slot_capacity"] == 128, "row wrapper capacity")
require(evidence["count_domains"]["physical_n_to_k_or_b_typed_join"] == "UNBOUND", "N/K/B join")
require(evidence["count_domains"]["k_row_stride_bytes"] == 6, "K row stride")
require(evidence["status"]["runtime_acceptance"] == "NOT_RUN", "runtime status")
joined = json.dumps(evidence, ensure_ascii=True) + (ROOT / "REPORT.md").read_text(encoding="utf-8")
require(re.search(r"[A-Za-z]:\\", joined) is None, "absolute path leak")
require(re.search(r"0x[0-9a-fA-F]{6,}", joined) is None, "raw native address leak")
require(manifest.get("contains_private_key_or_license") is False, "secret flag")
require(manifest.get("contains_pcm_or_media") is False, "media flag")
require(manifest.get("contains_native_source_or_decompilation") is False, "code flag")
print(f"PASS package={PACKAGE_ID} static_gate=1..128 wrapper=128 N_to_K_B=UNBOUND")
