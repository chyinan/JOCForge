from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "DEE-RATE-SETUP-ENUM-BOUNDARY-V1"
SOURCE_MANIFEST_SHA256 = "aa1a3298a4406c27e90dced75add919afe54b80c9edf401aca4cf2ae6afd26b0"
SOURCE_EVIDENCE_SHA256 = "bd2f80ec24ea12f87904d7d4646baadbb87130879f03aa70989b5f90b34d54ad"
EXPECTED = {
    "384": (0, 6),
    "448": (1, 8),
    "576": (2, 8),
    "640": (3, 8),
    "768": (4, 8),
    "1024": (5, 8),
}
PAYLOADS = ("EVIDENCE.json", "INPUT-ONLY-VECTORS.json", "REPORT.md", "SOURCE-MANIFESTS.json")


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(f"FAIL {PACKAGE_ID}: {message}")


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def build() -> None:
    rows = []
    for name in ("CHECK.py", *PAYLOADS):
        path = ROOT / name
        rows.append({"name": name, "bytes": path.stat().st_size, "sha256": sha256(path)})
    manifest = {
        "schema": "jocforge.wash_supplement_manifest.v1",
        "package_id": PACKAGE_ID,
        "recipient_role": "implementation-evidence-supplement",
        "status": "SIX_RATE_ENUM_VALUES_TO_TYPED_SETUP_BOUND; BSI_WRITER_JOIN_UNBOUND",
        "implementation_disposition": "Record the six static selector/setup rows only; do not claim runtime selection, BSI semantics, or all-rate E2E support.",
        "source_manifest_sha256": SOURCE_MANIFEST_SHA256,
        "source_evidence_sha256": SOURCE_EVIDENCE_SHA256,
        "contains_private_key_or_license": False,
        "contains_pcm_or_media": False,
        "contains_native_source_or_decompilation": False,
        "payloads": rows,
    }
    manifest_path = ROOT / "MANIFEST.json"
    manifest_path.write_text(json.dumps(manifest, indent=2, ensure_ascii=False) + "\n", encoding="utf-8", newline="\n")
    seal = {
        "schema": "jocforge.wash_supplement_seal.v1",
        "package_id": PACKAGE_ID,
        "status": "SEALED_MANUAL_HASH_AUDIT",
        "manifest_sha256": sha256(manifest_path),
        "payload_file_count": len(rows),
        "payload_hash_audit": f"PASS_{len(rows)}_OF_{len(rows)}",
        "checker_status": "PENDING_NORMAL_AND_PYTHON_O",
    }
    (ROOT / "package_seal.json").write_text(json.dumps(seal, indent=2) + "\n", encoding="utf-8", newline="\n")


def check() -> None:
    manifest_path = ROOT / "MANIFEST.json"
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    seal = json.loads((ROOT / "package_seal.json").read_text(encoding="utf-8"))
    require(manifest.get("package_id") == PACKAGE_ID, "package ID mismatch")
    require(manifest.get("source_manifest_sha256") == SOURCE_MANIFEST_SHA256, "source manifest mismatch")
    require(manifest.get("source_evidence_sha256") == SOURCE_EVIDENCE_SHA256, "source evidence mismatch")
    require(seal.get("manifest_sha256") == sha256(manifest_path), "manifest hash mismatch")
    require(seal.get("payload_file_count") == len(manifest.get("payloads", [])), "payload count mismatch")
    expected_names = {"CHECK.py", *PAYLOADS}
    actual_names = {row.get("name") for row in manifest.get("payloads", [])}
    require(actual_names == expected_names, "payload inventory mismatch")
    for row in manifest["payloads"]:
        path = ROOT / row["name"]
        require(path.is_file(), f"missing payload {row['name']}")
        require(path.stat().st_size == row["bytes"], f"byte count mismatch: {row['name']}")
        require(sha256(path) == row["sha256"], f"hash mismatch: {row['name']}")
    refs = json.loads((ROOT / "SOURCE-MANIFESTS.json").read_text(encoding="utf-8"))
    pair = next((x for x in refs["sources"] if x["package_id"] == "DEE-ADM-ATMOSMEZZ-RATE-PAIR-BSI-WRITER-WASH-V1"), None)
    require(pair is not None, "two-value parent receipt missing")
    require(pair["checksum_manifest_file_sha256"] == SOURCE_MANIFEST_SHA256, "parent manifest mismatch")
    require(pair["evidence_json_sha256"] == SOURCE_EVIDENCE_SHA256, "parent evidence mismatch")
    evidence = json.loads((ROOT / "EVIDENCE.json").read_text(encoding="utf-8"))
    actual = {v["xml_text"]: (v["selector_code"], int(v["typed_setup"].split("=")[-1])) for v in evidence["values"]}
    require(actual == EXPECTED, "six-value selector/setup mapping mismatch")
    require(evidence["limits"]["bsi_bitrate_field_identified"] is False, "BSI field must remain unbound")
    require(manifest.get("contains_private_key_or_license") is False, "secret-material flag must be false")
    require(manifest.get("contains_pcm_or_media") is False, "media flag must be false")
    require(manifest.get("contains_native_source_or_decompilation") is False, "native-code flag must be false")
    print(f"PASS package={PACKAGE_ID} payloads={len(expected_names)} six_values=BOUND bsi=UNBOUND runtime=NOT_RUN")


if __name__ == "__main__":
    if "--build" in sys.argv:
        build()
    else:
        check()
