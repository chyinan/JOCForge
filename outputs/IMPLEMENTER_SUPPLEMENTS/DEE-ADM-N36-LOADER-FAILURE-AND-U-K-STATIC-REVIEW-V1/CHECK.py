from __future__ import annotations

import hashlib
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "DEE-ADM-N36-LOADER-FAILURE-AND-U-K-STATIC-REVIEW-V1"
SOURCE_MANIFEST = "43bfe4193c1eaef42bfd758adf09d2a8e1e878e342d382c980fbd59356923df5"
SOURCE_SEAL = "37ea041a58dc9cd2f914fad52c954040670c3cf3436e5e9db76a4686f0b2d40f"
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

attempts = evidence["attempt_comparison"]
require(attempts["attempt05"]["target_component_loaded"] is True, "attempt05 control")
require(attempts["attempts06_to_10"]["target_component_loaded"] is False, "failed attempts boundary")
require(attempts["root_cause"] == "UNRESOLVED", "loader root cause")
loader = evidence["host_loader_static_boundary"]
require(loader["hash_bound_wrapper_count"] == 2, "host wrappers")
require(loader["attempt08_hook_error_value_reliable"] is False, "attempt08 error value")
chain = evidence["u_to_k_static_chain"]
require(chain["source_owner_slot_at_k_writer_entry"] == "NOT_INITIALIZED_YET", "owner init order")
require(chain["helper_call"].startswith("writer passes initialized"), "U helper input")
require(chain["runtime_owner_generation"] == "NOT_CAPTURED", "runtime boundary")
joined = json.dumps(evidence, ensure_ascii=True) + (ROOT / "REPORT.md").read_text(encoding="utf-8")
require(re.search(r"[A-Za-z]:\\", joined) is None, "absolute path leak")
require(re.search(r"0x[0-9a-fA-F]{6,}", joined) is None, "raw native address leak")
require(manifest.get("contains_private_key_or_license") is False, "secret flag")
require(manifest.get("contains_pcm_or_media") is False, "media flag")
require(manifest.get("contains_native_source_or_decompilation") is False, "code flag")
print(f"PASS package={PACKAGE_ID} static_U_to_K=BOUND runtime_join=NOT_CAPTURED")
