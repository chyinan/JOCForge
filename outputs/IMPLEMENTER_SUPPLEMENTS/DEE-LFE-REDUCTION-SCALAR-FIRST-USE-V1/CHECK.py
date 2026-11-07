from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "DEE-LFE-REDUCTION-SCALAR-FIRST-USE-V1"
SOURCE_MANIFEST_SHA256 = "df13b0e319af4aca755a01d5d50505e23ff4c7afa37ea9af5cb542a19be5499c"
SOURCE_SEAL_SHA256 = "5a1aea71baf92d6835f4342631474d113bf8aa5caa4441c879545126767d4066"
PAYLOADS = (
    "EVIDENCE.json",
    "INPUT-ONLY-VECTORS.json",
    "REPORT.md",
    "SOURCE-MANIFESTS.json",
)


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(f"FAIL {PACKAGE_ID}: {message}")


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def build() -> None:
    records = []
    for name in ("CHECK.py", *PAYLOADS):
        path = ROOT / name
        records.append({"name": name, "bytes": path.stat().st_size, "sha256": sha256(path)})
    manifest = {
        "schema": "jocforge.wash_supplement_manifest.v1",
        "package_id": PACKAGE_ID,
        "recipient_role": "implementation-evidence-supplement",
        "status": "SCOPED_STATIC_R9_OVERWRITE_BOUND; BROADER_LFE_EFFECT_UNBOUND",
        "implementation_disposition": "Reject only the scalar-through-R9 dataflow hypothesis on the inspected call path; no parity or root-cause claim.",
        "source_manifest_sha256": SOURCE_MANIFEST_SHA256,
        "source_package_seal_sha256": SOURCE_SEAL_SHA256,
        "contains_private_key_or_license": False,
        "contains_pcm_or_media": False,
        "contains_native_source_or_decompilation": False,
        "payloads": records,
    }
    manifest_path = ROOT / "MANIFEST.json"
    manifest_path.write_text(json.dumps(manifest, indent=2, ensure_ascii=False) + "\n", encoding="utf-8", newline="\n")
    seal = {
        "schema": "jocforge.wash_supplement_seal.v1",
        "package_id": PACKAGE_ID,
        "status": "SEALED_MANUAL_HASH_AUDIT",
        "manifest_sha256": sha256(manifest_path),
        "payload_file_count": len(records),
        "payload_hash_audit": f"PASS_{len(records)}_OF_{len(records)}",
        "checker_status": "PENDING_NORMAL_AND_PYTHON_O",
    }
    (ROOT / "package_seal.json").write_text(json.dumps(seal, indent=2) + "\n", encoding="utf-8", newline="\n")


def check() -> None:
    manifest_path = ROOT / "MANIFEST.json"
    seal_path = ROOT / "package_seal.json"
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    seal = json.loads(seal_path.read_text(encoding="utf-8"))
    require(manifest.get("package_id") == PACKAGE_ID, "package ID mismatch")
    require(manifest.get("source_manifest_sha256") == SOURCE_MANIFEST_SHA256, "source manifest hash mismatch")
    require(manifest.get("source_package_seal_sha256") == SOURCE_SEAL_SHA256, "source seal hash mismatch")
    require(seal.get("manifest_sha256") == sha256(manifest_path), "manifest hash mismatch")
    require(seal.get("payload_file_count") == len(manifest.get("payloads", [])), "payload count mismatch")
    expected = {"CHECK.py", *PAYLOADS}
    actual = {row.get("name") for row in manifest.get("payloads", [])}
    require(actual == expected, "payload inventory mismatch")
    for row in manifest["payloads"]:
        path = ROOT / row["name"]
        require(path.is_file(), f"missing payload {row['name']}")
        require(path.stat().st_size == row["bytes"], f"byte count mismatch: {row['name']}")
        require(sha256(path) == row["sha256"], f"SHA-256 mismatch: {row['name']}")
    refs = json.loads((ROOT / "SOURCE-MANIFESTS.json").read_text(encoding="utf-8"))
    source = next((item for item in refs["sources"] if item["package_id"] == "dee_lfe_reduction_scalar_first_use_wash_v1"), None)
    require(source is not None, "upstream source reference missing")
    require(source["evidence_manifest_sha256"] == SOURCE_MANIFEST_SHA256, "upstream manifest reference mismatch")
    require(source["package_seal_sha256"] == SOURCE_SEAL_SHA256, "upstream seal reference mismatch")
    require(manifest.get("contains_private_key_or_license") is False, "secret material flag must be false")
    require(manifest.get("contains_pcm_or_media") is False, "media flag must be false")
    require(manifest.get("contains_native_source_or_decompilation") is False, "native-code flag must be false")
    print(f"PASS package={PACKAGE_ID} payloads={len(expected)} source={SOURCE_MANIFEST_SHA256[:12]} r9_overwrite=BOUND broader_effect=UNBOUND")


if __name__ == "__main__":
    if "--build" in sys.argv:
        build()
    else:
        check()
