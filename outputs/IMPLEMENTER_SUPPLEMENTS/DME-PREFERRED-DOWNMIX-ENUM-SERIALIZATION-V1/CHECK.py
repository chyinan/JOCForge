from __future__ import annotations

import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "DME-PREFERRED-DOWNMIX-ENUM-SERIALIZATION-V1"
UPSTREAM_MANIFEST = "c30e7eb3c4577ff81d25105e0e1011379d15431068e01e918ba96e759a359eb5"
EXPECTED = {
    "CHECK.py", "EVIDENCE.json", "INPUT-ONLY-VECTORS.json", "REPORT.md",
    "SOURCE-MANIFESTS.json", "UPSTREAM-ENUM-SERIALIZATION.json",
    "UPSTREAM-INPUT-ONLY-VECTORS.json", "UPSTREAM-MANIFEST.json",
    "UPSTREAM-SOURCE-MANIFESTS.json",
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
up_enum = json.loads((ROOT / "UPSTREAM-ENUM-SERIALIZATION.json").read_text(encoding="utf-8"))
up_vectors = json.loads((ROOT / "UPSTREAM-INPUT-ONLY-VECTORS.json").read_text(encoding="utf-8"))

require(manifest.get("schema") == "jocforge.wash_supplement_manifest.v1", "manifest schema")
require(manifest.get("package_id") == PACKAGE_ID, "manifest package id")
require(manifest.get("source_manifest_sha256") == UPSTREAM_MANIFEST, "source manifest binding")
require(manifest.get("contains_private_key_or_license") is False, "secret boundary")
require(manifest.get("contains_pcm_or_media") is False, "media boundary")
require(manifest.get("contains_native_source_or_decompilation") is False, "source boundary")
require(seal.get("schema") == "jocforge.wash_supplement_seal.v1", "seal schema")
require(seal.get("package_id") == PACKAGE_ID, "seal package id")
require(seal.get("manifest_sha256") == sha(ROOT / "MANIFEST.json"), "manifest seal hash")
payloads = manifest.get("payloads")
require(isinstance(payloads, list) and {row.get("name") for row in payloads} == EXPECTED, "payload inventory")
for row in payloads:
    path = ROOT / row["name"]
    require(path.is_file(), f"missing payload {row['name']}")
    require(path.stat().st_size == row["bytes"], f"byte length {row['name']}")
    require(sha(path) == row["sha256"], f"payload hash {row['name']}")
require(seal.get("payload_file_count") == len(EXPECTED), "payload count")
require(seal.get("payload_hash_audit") == f"PASS_{len(EXPECTED)}_OF_{len(EXPECTED)}", "payload audit")

require(sha(ROOT / "UPSTREAM-MANIFEST.json") == UPSTREAM_MANIFEST, "upstream manifest hash")
require(up_manifest.get("package") == "F13C-preferred-downmix-enum-serialization-addendum-v1", "upstream package id")
require(up_manifest.get("source_app_sha256") == "c51aa127eb0d18b9bd78f7189b681ab5209efd87a9c16b3f48ed2e1cd1649917", "DME source hash")
require(up_manifest.get("enum_value_count") == 3, "enum count")
require(up_manifest.get("fourth_explicit_enum_value") == "NOT_PRESENT_IN_HASHED_DME_HOST_SCHEMA_OR_HANDLER", "fourth enum status")
require(evidence.get("status", {}).get("dme_host_enum_values") == "BOUND_STATIC_THREE_VALUES", "enum evidence")
require(evidence.get("status", {}).get("dee_native_parser_and_typed_consumer") == "UNBOUND", "native consumer boundary")
require(evidence.get("status", {}).get("runtime_user_value") == "NOT_CAPTURED", "runtime value boundary")
require(vectors.get("execution") == "NOT_RUN", "vector execution")
require(len(vectors.get("cases", [])) == 4, "vector cases")
require(sources.get("upstream_formal_seal") == "NOT_PRESENT", "upstream seal disposition")
require(up_enum.get("property_identity", {}).get("enum_value_count") == 3, "upstream enum count")
require(up_enum.get("property_identity", {}).get("fourth_explicit_enum_value") == "NOT_PRESENT_IN_HASHED_DME_HOST_SCHEMA_OR_HANDLER", "upstream fourth enum")
require(up_vectors.get("execution") == "NOT_RUN", "upstream vector execution")
print(f"PASS {PACKAGE_ID} values=3 source={UPSTREAM_MANIFEST[:12]} native_effect=UNBOUND")

