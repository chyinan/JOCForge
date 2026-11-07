from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "DEE-ADM-U-GROUP-K-SIX-BYTE-ROW-WRITER-V1"
SOURCE_MANIFEST_SHA256 = "cf865e0eb2617a25a8d2619b9a381c80b8aff8034ca90854f7ad8d1bc86a65cf"
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
        "status": "U_GROUP_ENTRY_TO_K_ROW_BOUND; ROW_SEMANTIC_AND_BYTE3_INIT_PARTIAL",
        "implementation_disposition": "Use exact row offsets and source fields; preserve byte+3 as not evaluated and semantics as partial.",
        "source_manifest_sha256": SOURCE_MANIFEST_SHA256,
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
    require(seal.get("manifest_sha256") == sha256(manifest_path), "manifest hash mismatch")
    require(seal.get("payload_file_count") == len(manifest.get("payloads", [])), "payload count mismatch")
    expected = {"CHECK.py", *PAYLOADS}
    actual = {row.get("name") for row in manifest.get("payloads", [])}
    require(actual == expected, "payload inventory mismatch")
    for row in manifest["payloads"]:
        path = ROOT / row["name"]
        require(path.is_file(), f"missing payload {row['name']}")
        require(path.stat().st_size == row["bytes"], f"byte count mismatch: {row['name']}")
        require(sha256(path) == row["sha256"], f"hash mismatch: {row['name']}")
    evidence = json.loads((ROOT / "EVIDENCE.json").read_text(encoding="utf-8"))
    require(evidence["source_package"]["manifest_sha256"] == SOURCE_MANIFEST_SHA256, "source manifest mismatch")
    require(evidence["status"]["accepted_group_entry_to_K_row"] == "STATICALLY_BOUND_ONE_ROW_PER_ACCEPTED_ENTRY", "row mapping mismatch")
    require(evidence["status"]["row_byte_plus_0x03_store_or_initialization"] == "NOT_EVALUATED", "byte+3 must remain uninitialized/unknown")
    require(evidence["status"]["universal_ADM_admission"] == "NOT_ESTABLISHED", "universal admission must remain unproven")
    require(manifest.get("contains_private_key_or_license") is False, "secret-material flag must be false")
    require(manifest.get("contains_pcm_or_media") is False, "media flag must be false")
    require(manifest.get("contains_native_source_or_decompilation") is False, "native-code flag must be false")
    print(f"PASS package={PACKAGE_ID} payloads={len(expected)} K_row_layout=BOUND byte3=NOT_EVALUATED")


if __name__ == "__main__":
    if "--build" in sys.argv:
        build()
    else:
        check()
