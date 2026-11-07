from __future__ import annotations

import hashlib
import json
import math
import struct
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "DEE-LFE-RECURRENCE-COEFFICIENT-VECTORS-V1"
UPSTREAM_MANIFEST = "f41ca962188d127bf7c723d566862adbbe586979bad95f1d18d843f0f6221b60"
UPSTREAM_SEAL = "dfeb0ef58b293a568209b65ed32edeb1f7a28e8bbb10e1e656773be8d3acf28a"
UPSTREAM_IDENTITIES = "d6e12c8843ccde2ce994b25fb18aed5f4a11b7ab200edf3615781a6a4e7414e4"
IMAGE_SHA256 = "0cf23520799f52323b0994e5fcc16924da7b027a5b83a0cff3b4f06792c18532"
EXPECTED = {
    "zero_branch_table_a": ("0x140628c20", 128),
    "zero_branch_table_b": ("0x140628e20", 128),
    "zero_branch_shared_scalars": ("0x140616dd0", 4),
    "nonzero_branch_table_a": ("0x140629020", 64),
    "nonzero_branch_table_b": ("0x140628820", 64),
}


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(f"FAIL: {message}")


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


manifest = json.loads((ROOT / "MANIFEST.json").read_text(encoding="utf-8"))
seal = json.loads((ROOT / "package_seal.json").read_text(encoding="utf-8"))
evidence = json.loads((ROOT / "EVIDENCE.json").read_text(encoding="utf-8"))
vectors = json.loads((ROOT / "INPUT-ONLY-VECTORS.json").read_text(encoding="utf-8"))
sources = json.loads((ROOT / "SOURCE-MANIFESTS.json").read_text(encoding="utf-8"))
up_manifest = json.loads((ROOT / "UPSTREAM-EVIDENCE-MANIFEST.json").read_text(encoding="utf-8"))
up_seal = json.loads((ROOT / "UPSTREAM-PACKAGE-SEAL.json").read_text(encoding="utf-8"))
up_ids = json.loads((ROOT / "UPSTREAM-SOURCE-IDENTITIES.json").read_text(encoding="utf-8"))

require(manifest.get("schema") == "jocforge.wash_supplement_manifest.v1", "manifest schema")
require(manifest.get("package_id") == PACKAGE_ID, "manifest package id")
require(manifest.get("source_manifest_sha256") == UPSTREAM_MANIFEST, "upstream manifest binding")
require(manifest.get("contains_private_key_or_license") is False, "secret boundary")
require(manifest.get("contains_pcm_or_media") is False, "media boundary")
require(manifest.get("contains_native_source_or_decompilation") is False, "native source boundary")
require(seal.get("package_id") == PACKAGE_ID, "seal package id")
require(seal.get("manifest_sha256") == sha256(ROOT / "MANIFEST.json"), "manifest seal hash")

payloads = manifest.get("payloads")
require(isinstance(payloads, list) and payloads, "payload list")
expected_names = {
    "CHECK.py", "CONSTANTS-SOURCE.md", "EVIDENCE.json", "INPUT-ONLY-VECTORS.json",
    "REPORT.md", "SOURCE-MANIFESTS.json", "UPSTREAM-EVIDENCE-MANIFEST.json",
    "UPSTREAM-PACKAGE-SEAL.json", "UPSTREAM-SOURCE-IDENTITIES.json",
}
require({item.get("name") for item in payloads} == expected_names, "payload set")
for item in payloads:
    path = ROOT / item["name"]
    require(path.is_file(), f"payload missing: {item['name']}")
    require(path.stat().st_size == item["bytes"], f"payload size: {item['name']}")
    require(sha256(path) == item["sha256"], f"payload hash: {item['name']}")
require(seal.get("payload_file_count") == len(payloads), "seal payload count")
require(seal.get("payload_hash_audit") == f"PASS_{len(payloads)}_OF_{len(payloads)}", "seal payload audit")

require(sha256(ROOT / "UPSTREAM-EVIDENCE-MANIFEST.json") == UPSTREAM_MANIFEST, "upstream manifest file")
require(sha256(ROOT / "UPSTREAM-PACKAGE-SEAL.json") == UPSTREAM_SEAL, "upstream seal file")
require(sha256(ROOT / "UPSTREAM-SOURCE-IDENTITIES.json") == UPSTREAM_IDENTITIES, "upstream identities file")
require(up_manifest.get("package_id") == "dee_lfe_recurrence_coefficients_input_only_wash_v1", "upstream package id")
require(up_manifest.get("target_image", {}).get("sha256") == IMAGE_SHA256, "target image hash binding")
require(up_manifest.get("runtime_observation") == "NOT_RUN", "runtime boundary")
require(up_manifest.get("numeric_parity") == "NOT_RUN", "parity boundary")
require(up_manifest.get("root_cause") == "NOT_ESTABLISHED", "root cause boundary")
require(up_manifest.get("selector") == "UNBOUND", "selector boundary")
require(up_manifest.get("sample_lfe_branch") == "UNBOUND", "sample branch boundary")
require(up_ids.get("target_image", {}).get("sha256") == IMAGE_SHA256, "source identity image hash")
require(up_ids.get("vector_payload", {}).get("sha256") == sha256(ROOT / "INPUT-ONLY-VECTORS.json"), "vector file binding")
require(sources.get("upstream_evidence_manifest_sha256") == UPSTREAM_MANIFEST, "source manifest ref")
require(sources.get("upstream_package_seal_sha256") == UPSTREAM_SEAL, "source seal ref")
require(sources.get("upstream_source_identities_sha256") == UPSTREAM_IDENTITIES, "source identities ref")
require(sources.get("target_image", {}).get("sha256") == IMAGE_SHA256, "source target image")

require(vectors.get("schema") == "dee-lfe-recurrence-coefficients-input-only.v1", "vector schema")
require(vectors.get("source_image_sha256") == IMAGE_SHA256, "vector image hash")
require(vectors.get("data_vectors_only") is True, "data-only marker")
require(vectors.get("access_policy", {}).get("target_code_exported") is False, "code export")
require(vectors.get("access_policy", {}).get("decompilation_exported") is False, "decompilation export")
require(vectors.get("access_policy", {}).get("binary_exported") is False, "binary export")
require(vectors.get("access_policy", {}).get("pcm_or_payload_read") is False, "PCM/payload boundary")
require(vectors.get("access_policy", {}).get("key_or_license_read") is False, "key/license boundary")
require(vectors.get("branch_and_parity_binding") == {
    "recurrence_selector": "UNBOUND",
    "sample_lfe_branch": "UNBOUND",
    "numeric_parity": "NOT_RUN",
    "root_cause": "NOT_ESTABLISHED",
}, "branch/parity status")

rows = {row.get("name"): row for row in vectors.get("vectors", [])}
require(set(rows) == set(EXPECTED), "vector names")
total = 0
for name, (base_text, count) in EXPECTED.items():
    row = rows[name]
    base = int(base_text, 16)
    require(int(row["base_address"], 16) == base, f"base address: {name}")
    require(row["element_count"] == count, f"element count: {name}")
    require(row["byte_count"] == count * 4, f"byte count: {name}")
    require(row["element_type"] == "IEEE-754 binary32", f"element type: {name}")
    require(row["byte_order"] == "little-endian", f"byte order: {name}")
    require(row["memory_block_name"] == ".rdata" and row["memory_block_executable"] is False,
            f"memory block: {name}")
    elements = row.get("elements")
    require(isinstance(elements, list) and len(elements) == count, f"element list: {name}")
    for index, element in enumerate(elements):
        require(element["index"] == index, f"index: {name}[{index}]")
        require(int(element["address"], 16) == base + 4 * index, f"address: {name}[{index}]")
        raw = element["raw_word_le_hex"]
        require(isinstance(raw, str) and len(raw) == 10 and raw.startswith("0x"),
                f"raw word: {name}[{index}]")
        word = int(raw[2:], 16).to_bytes(4, "little")
        value = struct.unpack("<f", word)[0]
        declared = element["f32_value"]
        if math.isnan(value):
            require(declared == "NaN", f"NaN value: {name}[{index}]")
        elif value == math.inf:
            require(declared == "+Infinity", f"infinite value: {name}[{index}]")
        elif value == -math.inf:
            require(declared == "-Infinity", f"infinite value: {name}[{index}]")
        else:
            require(isinstance(declared, (int, float)), f"numeric value: {name}[{index}]")
            require(struct.pack("<f", float(declared)) == word, f"raw/F32 mismatch: {name}[{index}]")
        total += 1

require(total == 388, "total vector elements")
require(evidence.get("status", {}).get("numeric_parity") == "NOT_RUN", "evidence parity status")
require(evidence.get("status", {}).get("sample_lfe_branch") == "UNBOUND", "evidence sample branch")
print(f"PASS {PACKAGE_ID} vectors={len(rows)} elements={total} source={UPSTREAM_MANIFEST[:12]} parity=NOT_RUN")

