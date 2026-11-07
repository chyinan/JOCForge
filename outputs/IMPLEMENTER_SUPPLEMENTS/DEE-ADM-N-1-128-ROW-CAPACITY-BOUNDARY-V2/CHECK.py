from __future__ import annotations

import hashlib
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "DEE-ADM-N-1-128-ROW-CAPACITY-BOUNDARY-V2"
SOURCE_MANIFEST = "79ac8fc623f6078e130795ce07de5493021f67fdef48e39a429f9f0ec6325e28"
SOURCE_SEAL = "bee8ad474d639af7193c615fa10c1c976083152cb7bd0d6e131bda3661b238e4"
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

require(evidence["physical_n_gate"]["static_accepted_range"] == {"minimum": 1, "maximum": 128}, "N gate range")
require(evidence["first_row_wrapper"]["slot_capacity"] == 128, "wrapper capacity")
require(evidence["first_row_wrapper"]["receives_n_as_allocation_count"] is False, "wrapper N argument")
require(evidence["count_domain_joins"]["physical_n_to_k"] == "UNBOUND", "N to K")
require(evidence["count_domain_joins"]["physical_n_to_b"] == "UNBOUND", "N to B")
require(all(row["evidence_grade"] == "INPUT_ONLY_MODEL_PROJECTION_NOT_NATIVE_CAPTURE" for row in evidence["input_only_projections"]), "sample projection grade")
joined = json.dumps(evidence, ensure_ascii=True) + (ROOT / "REPORT.md").read_text(encoding="utf-8")
require(re.search(r"[A-Za-z]:\\", joined) is None, "absolute path leak")
require(re.search(r"0x[0-9a-fA-F]{6,}", joined) is None, "raw native address leak")
require(manifest.get("contains_private_key_or_license") is False, "secret flag")
require(manifest.get("contains_pcm_or_media") is False, "media flag")
require(manifest.get("contains_native_source_or_decompilation") is False, "code flag")
print(f"PASS package={PACKAGE_ID} N_gate=1..128 wrapper=128 N_to_K_B=UNBOUND")
