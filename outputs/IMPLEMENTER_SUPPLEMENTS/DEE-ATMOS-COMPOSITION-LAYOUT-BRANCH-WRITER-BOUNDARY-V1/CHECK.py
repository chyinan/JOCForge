from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "DEE-ATMOS-COMPOSITION-LAYOUT-BRANCH-WRITER-BOUNDARY-V1"
SOURCE_MANIFEST_SHA256 = "fa60ca124c7dcbd694ef64828277cf864f0700b11231acbf97a0d184d3f089d8"
SOURCE_CHECKSUM_FILE_SHA256 = "d3848abf82fcd83f8a010646103d2b34df68fefe2725c555a29a06bdbebab2ff"
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
        "status": "LAYOUT_BRANCH_STATIC_WRITER_BOUND; RUNTIME_SELECTION_NOT_CAPTURED",
        "implementation_disposition": "Record the static same-owner flag rule only; do not infer sample selection or audio effect.",
        "source_manifest_sha256": SOURCE_MANIFEST_SHA256,
        "source_checksum_file_sha256": SOURCE_CHECKSUM_FILE_SHA256,
        "contains_private_key_or_license": False,
        "contains_pcm_or_media": False,
        "contains_native_source_or_decompilation": False,
        "payloads": rows,
    }
    path = ROOT / "MANIFEST.json"
    path.write_text(json.dumps(manifest, indent=2, ensure_ascii=False) + "\n", encoding="utf-8", newline="\n")
    seal = {
        "schema": "jocforge.wash_supplement_seal.v1",
        "package_id": PACKAGE_ID,
        "status": "SEALED_MANUAL_HASH_AUDIT",
        "manifest_sha256": sha256(path),
        "payload_file_count": len(rows),
        "payload_hash_audit": f"PASS_{len(rows)}_OF_{len(rows)}",
        "checker_status": "PENDING_NORMAL_AND_PYTHON_O",
    }
    (ROOT / "package_seal.json").write_text(json.dumps(seal, indent=2) + "\n", encoding="utf-8", newline="\n")


def check() -> None:
    manifest_path = ROOT / "MANIFEST.json"
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    seal = json.loads((ROOT / "package_seal.json").read_text(encoding="utf-8"))
    evidence = json.loads((ROOT / "EVIDENCE.json").read_text(encoding="utf-8"))
    require(manifest.get("package_id") == PACKAGE_ID, "package ID mismatch")
    require(manifest.get("source_manifest_sha256") == SOURCE_MANIFEST_SHA256, "source manifest mismatch")
    require(manifest.get("source_checksum_file_sha256") == SOURCE_CHECKSUM_FILE_SHA256, "source checksum file mismatch")
    require(seal.get("manifest_sha256") == sha256(manifest_path), "manifest hash mismatch")
    expected = {"CHECK.py", *PAYLOADS}
    require({row.get("name") for row in manifest.get("payloads", [])} == expected, "payload inventory mismatch")
    require(seal.get("payload_file_count") == len(expected), "payload count mismatch")
    for row in manifest["payloads"]:
        path = ROOT / row["name"]
        require(path.is_file(), f"missing {row['name']}")
        require(path.stat().st_size == row["bytes"], f"size mismatch {row['name']}")
        require(sha256(path) == row["sha256"], f"hash mismatch {row['name']}")
    require(evidence["source_package"]["manifest_sha256"] == SOURCE_MANIFEST_SHA256, "evidence source manifest mismatch")
    require(evidence["status"]["runtime_branch_value"] == "NOT_CAPTURED", "runtime value must remain not captured")
    require(evidence["status"]["global_absence_claim"] is False, "global absence claim forbidden")
    require(manifest.get("contains_private_key_or_license") is False, "secret flag")
    require(manifest.get("contains_pcm_or_media") is False, "media flag")
    require(manifest.get("contains_native_source_or_decompilation") is False, "native code flag")
    print(f"PASS package={PACKAGE_ID} static_writer=BOUND runtime_branch=NOT_CAPTURED")


if __name__ == "__main__":
    if "--build" in sys.argv:
        build()
    else:
        check()
