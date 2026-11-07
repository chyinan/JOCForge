from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "DEE-RATE-PAIR-BSI-WRITER-BOUNDARY-V1"
SOURCE_MANIFEST_SHA256 = "aa1a3298a4406c27e90dced75add919afe54b80c9edf401aca4cf2ae6afd26b0"
SOURCE_EVIDENCE_SHA256 = "bd2f80ec24ea12f87904d7d4646baadbb87130879f03aa70989b5f90b34d54ad"
PAYLOADS = ("EVIDENCE.json", "INPUT-ONLY-VECTORS.json", "REPORT.md", "SOURCE-MANIFESTS.json")


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
        "status": "TWO_RATE_VALUES_TO_TYPED_SETUP_BOUND; BSI_FIELD_AND_WRITER_OWNER_UNBOUND",
        "implementation_disposition": "Record only the two selected rate values, typed setup values, and exact writer first-break; do not claim complete bitrate support.",
        "source_manifest_sha256": SOURCE_MANIFEST_SHA256,
        "source_evidence_sha256": SOURCE_EVIDENCE_SHA256,
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
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    seal = json.loads((ROOT / "package_seal.json").read_text(encoding="utf-8"))
    require(manifest.get("package_id") == PACKAGE_ID, "package ID mismatch")
    require(manifest.get("source_manifest_sha256") == SOURCE_MANIFEST_SHA256, "source manifest hash mismatch")
    require(manifest.get("source_evidence_sha256") == SOURCE_EVIDENCE_SHA256, "source evidence hash mismatch")
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
    source = next((item for item in refs["sources"] if item["package_id"] == "DEE-ADM-ATMOSMEZZ-RATE-PAIR-BSI-WRITER-WASH-V1"), None)
    require(source is not None, "upstream source reference missing")
    require(source["checksum_manifest_file_sha256"] == SOURCE_MANIFEST_SHA256, "upstream checksum manifest mismatch")
    require(source["evidence_sha256"] == SOURCE_EVIDENCE_SHA256, "upstream evidence hash mismatch")
    require(manifest.get("contains_private_key_or_license") is False, "secret-material flag must be false")
    require(manifest.get("contains_pcm_or_media") is False, "media flag must be false")
    require(manifest.get("contains_native_source_or_decompilation") is False, "native-code flag must be false")
    print(f"PASS package={PACKAGE_ID} payloads={len(expected)} source={SOURCE_MANIFEST_SHA256[:12]} 384=6 1024=8 bsi=UNBOUND")


if __name__ == "__main__":
    if "--build" in sys.argv:
        build()
    else:
        check()
