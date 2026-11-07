from __future__ import annotations

import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "DEE-WAV-SELECTOR3-READER-EOF-BOUNDARY-V1"
UPSTREAM_MANIFEST = "cf989438b3d4bc1c0d691504dc521e68881a2458bb27b4cc85391f1a36ac791a"
UPSTREAM_SEAL = "58a81486835f87be5a3535490d79551884a2772c8890042431e57760afe8c489"
UPSTREAM_VECTOR = "f6630dc5cb0e95bb3c11e3c10783ae148c02c1e3dd1b05d22c3001cc27fd77fd"
UPSTREAM_SUMS = "cd5fc6ff23c4df0931b46d63fb3ba29332d55bd3a846303b7cebf0d90092a0bc"
EXPECTED = {
    "CHECK.py", "EVIDENCE.json", "INPUT-ONLY-VECTORS.json", "REPORT.md",
    "SOURCE-MANIFESTS.json", "UPSTREAM-INPUT-VECTOR.json", "UPSTREAM-MANIFEST.json",
    "UPSTREAM-PACKAGE-SEAL.json", "UPSTREAM-SHA256SUMS.txt",
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
source = json.loads((ROOT / "SOURCE-MANIFESTS.json").read_text(encoding="utf-8"))
up_manifest = json.loads((ROOT / "UPSTREAM-MANIFEST.json").read_text(encoding="utf-8"))
up_vector = json.loads((ROOT / "UPSTREAM-INPUT-VECTOR.json").read_text(encoding="utf-8"))

require(manifest.get("schema") == "jocforge.wash_supplement_manifest.v1", "manifest schema")
require(manifest.get("package_id") == PACKAGE_ID, "manifest package id")
require(manifest.get("source_manifest_sha256") == UPSTREAM_MANIFEST, "source manifest binding")
require(manifest.get("contains_private_key_or_license") is False, "secret boundary")
require(manifest.get("contains_pcm_or_media") is False, "media boundary")
require(manifest.get("contains_native_source_or_decompilation") is False, "source boundary")
require(seal.get("schema") == "jocforge.wash_supplement_seal.v1", "seal schema")
require(seal.get("package_id") == PACKAGE_ID, "seal package id")
require(seal.get("manifest_sha256") == sha(ROOT / "MANIFEST.json"), "manifest seal")
payloads = manifest.get("payloads")
require(isinstance(payloads, list) and {p.get("name") for p in payloads} == EXPECTED, "payload inventory")
for item in payloads:
    path = ROOT / item["name"]
    require(path.is_file(), f"payload missing: {item['name']}")
    require(path.stat().st_size == item["bytes"], f"payload size: {item['name']}")
    require(sha(path) == item["sha256"], f"payload hash: {item['name']}")
require(seal.get("payload_file_count") == len(EXPECTED), "seal count")
require(seal.get("payload_hash_audit") == f"PASS_{len(EXPECTED)}_OF_{len(EXPECTED)}", "seal audit")

require(sha(ROOT / "UPSTREAM-MANIFEST.json") == UPSTREAM_MANIFEST, "upstream manifest hash")
require(sha(ROOT / "UPSTREAM-PACKAGE-SEAL.json") == UPSTREAM_SEAL, "upstream seal hash")
require(sha(ROOT / "UPSTREAM-INPUT-VECTOR.json") == UPSTREAM_VECTOR, "upstream vector hash")
require(sha(ROOT / "UPSTREAM-SHA256SUMS.txt") == UPSTREAM_SUMS, "upstream checksum-list hash")
require(up_manifest.get("status") == "WAV_SELECTOR_BOUND_CONCRETE_CALLBACK_AND_EOF_READ_UNBOUND", "upstream status")
require(up_manifest.get("provider_dispatch", {}).get("selector") == 3, "WAV selector")
require(up_manifest.get("provider_dispatch", {}).get("provider_open_callback_target") == "UNBOUND", "provider callback boundary")
require(up_manifest.get("disposition", {}).get("end_sample_minus_one_read") == "NOT_CAPTURED", "last-sample boundary")
require(up_vector.get("ghidra", {}).get("version") == "12.2", "Ghidra version")
require(up_vector.get("not_run", {}).get("dee") is True, "DEE execution boundary")
require(evidence.get("status", {}).get("concrete_callback") == "UNBOUND_RUNTIME_REGISTRY_CALLBACK", "callback boundary")
require(evidence.get("status", {}).get("endSample_minus_one_read") == "NOT_CAPTURED", "last-sample status")
require(vectors.get("provider_callback_target") == "UNBOUND", "vector callback boundary")
require(vectors.get("raw_audio_or_payload_included") is False, "vector payload boundary")
print(f"PASS {PACKAGE_ID} selector=3 callback=UNBOUND EOF=NOT_CAPTURED")

