from __future__ import annotations

import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "DEE-COMPOSITE-SELECTOR-DIRECT-INPUT-SEMANTICS-V1"
UPSTREAM_MANIFEST = "47f91ba7eb4efacb5ad19202631d508a8b39b26d1e14547ce926521998141ff4"
IMAGE_SHA256 = "3d66bcec36031fd48e6565d15f05fea656642377ca4f8c98cdce1cce8b7e95d2"
EXPECTED = {
    "CHECK.py", "EVIDENCE.json", "INPUT-ONLY-VECTORS.json", "REPORT.md",
    "SOURCE-MANIFESTS.json", "UPSTREAM-CHECK.json", "UPSTREAM-FUNCTION-HASHES.json",
    "UPSTREAM-INVOCATION.json", "UPSTREAM-MANIFEST.json",
    "UPSTREAM-SELECTOR-INPUT-SEMANTICS.json", "UPSTREAM-SOURCE-LINKS.json",
}


def require(ok: bool, message: str) -> None:
    if not ok:
        raise SystemExit(f"FAIL: {message}")


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


manifest = json.loads((ROOT / "MANIFEST.json").read_text(encoding="utf-8"))
seal = json.loads((ROOT / "package_seal.json").read_text(encoding="utf-8"))
evidence = json.loads((ROOT / "EVIDENCE.json").read_text(encoding="utf-8"))
vectors = json.loads((ROOT / "INPUT-ONLY-VECTORS.json").read_text(encoding="utf-8"))
sources = json.loads((ROOT / "SOURCE-MANIFESTS.json").read_text(encoding="utf-8"))
up_manifest = json.loads((ROOT / "UPSTREAM-MANIFEST.json").read_text(encoding="utf-8"))
up_selector = json.loads((ROOT / "UPSTREAM-SELECTOR-INPUT-SEMANTICS.json").read_text(encoding="utf-8"))
up_functions = json.loads((ROOT / "UPSTREAM-FUNCTION-HASHES.json").read_text(encoding="utf-8"))

require(manifest.get("schema") == "jocforge.wash_supplement_manifest.v1", "manifest schema")
require(manifest.get("package_id") == PACKAGE_ID, "manifest package id")
require(manifest.get("source_manifest_sha256") == UPSTREAM_MANIFEST, "source hash binding")
require(manifest.get("contains_private_key_or_license") is False, "secret boundary")
require(manifest.get("contains_pcm_or_media") is False, "media boundary")
require(manifest.get("contains_native_source_or_decompilation") is False, "source boundary")
require(seal.get("schema") == "jocforge.wash_supplement_seal.v1", "seal schema")
require(seal.get("package_id") == PACKAGE_ID, "seal package id")
require(seal.get("manifest_sha256") == sha(ROOT / "MANIFEST.json"), "manifest seal hash")

payloads = manifest.get("payloads")
require(isinstance(payloads, list) and {p.get("name") for p in payloads} == EXPECTED, "payload set")
for row in payloads:
    path = ROOT / row["name"]
    require(path.is_file(), f"missing payload: {row['name']}")
    require(path.stat().st_size == row["bytes"], f"payload byte count: {row['name']}")
    require(sha(path) == row["sha256"], f"payload hash: {row['name']}")
require(seal.get("payload_file_count") == len(EXPECTED), "seal payload count")
require(seal.get("payload_hash_audit") == f"PASS_{len(EXPECTED)}_OF_{len(EXPECTED)}", "seal payload audit")

require(sha(ROOT / "UPSTREAM-MANIFEST.json") == UPSTREAM_MANIFEST, "upstream manifest hash")
require(up_manifest.get("package_id") == "COMPOSITE-SELECTOR-INPUT-SEMANTICS-V1", "upstream package id")
require(up_manifest.get("target_image_sha256") == IMAGE_SHA256, "upstream image hash")
require(up_selector.get("target_image_sha256") == IMAGE_SHA256, "selector image hash")
require(up_selector.get("scope", {}).get("new_ghidra_query_this_turn") == "NOT_RUN", "upstream run status")
require(up_selector.get("selector", {}).get("runtime_value") == "NOT_CAPTURED", "runtime value boundary")
require(up_selector.get("direct_input_1_option_lookup", {}).get("mapped_option_payload_read") is False, "key-2 payload boundary")
require(up_selector.get("direct_input_2_source_descriptor_comparison", {}).get("actual_descriptor_list_owner_or_generation") == "UNBOUND", "descriptor owner boundary")
require(up_functions.get("selected_image_sha256") == IMAGE_SHA256, "function image binding")
require(sources.get("upstream_manifest_sha256") == UPSTREAM_MANIFEST, "source record manifest")
require(sources.get("upstream_formal_seal") == "NOT_PRESENT", "source seal disposition")
require(evidence.get("status", {}).get("selected_runtime_mode") == "NOT_CAPTURED", "runtime mode")
require(evidence.get("status", {}).get("source_descriptor_owner_generation") == "UNBOUND", "descriptor owner")
require(evidence.get("status", {}).get("key2_mapped_payload_semantics") == "UNBOUND", "key2 payload semantics")
require(evidence.get("runtime_and_safety", {}).get("dee_run") is False, "DEE boundary")
require(evidence.get("runtime_and_safety", {}).get("pcm_or_payload_read") is False, "payload boundary")
require(vectors.get("vector_scope") == "SYMBOLIC_PREDICATE_TRUTH_TABLE_NOT_SAMPLE", "vector scope")
cases = vectors.get("cases")
require(isinstance(cases, list) and len(cases) == 4, "symbolic cases")
for case in cases:
    want = int(bool(case["key2_lookup_nonzero"]) or bool(case["descriptor_comparison_nonzero"]))
    require(case["selector_T_plus_0x100"] == want, "OR truth table")
print(f"PASS {PACKAGE_ID} payloads={len(payloads)} source={UPSTREAM_MANIFEST[:12]} sample_mode=NOT_CAPTURED")

