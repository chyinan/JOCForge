from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "DEE-PREFERRED-DOWNMIX-CONFIG-TREE-WRITER-BOUNDARY-V1"
SOURCE_MANIFEST_SHA256 = "eab045aee24d204d1a7b576aff79ec3c42d90d6db3af7bf5b244e702d5f983b4"
SOURCE_EVIDENCE_SHA256 = "31f733861d99c776ed3f92a3eaf5eb914792daffeecb45075ac71a4839c7b58c"
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
        "status": "CONFIG_TREE_OPTION_WRITER_BOUND; TYPED_ENCODER_STORAGE_JOIN_UNBOUND",
        "implementation_disposition": "Register the intermediate record/config-tree write; preserve the absent typed edge to encoder storage.",
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
    expected = {"CHECK.py", *PAYLOADS}
    actual = {row.get("name") for row in manifest.get("payloads", [])}
    require(actual == expected, "payload inventory mismatch")
    for row in manifest["payloads"]:
        path = ROOT / row["name"]
        require(path.is_file(), f"missing payload {row['name']}")
        require(path.stat().st_size == row["bytes"], f"byte count mismatch: {row['name']}")
        require(sha256(path) == row["sha256"], f"hash mismatch: {row['name']}")
    evidence = json.loads((ROOT / "EVIDENCE.json").read_text(encoding="utf-8"))
    require(evidence["source_package"]["manifest_sha256"] == SOURCE_MANIFEST_SHA256, "upstream manifest mismatch")
    require(evidence["source_package"]["evidence_sha256"] == SOURCE_EVIDENCE_SHA256, "upstream evidence mismatch")
    require(evidence["status"]["config_tree_to_this_plus_0x360_writer"] == "UNBOUND_STATIC", "typed storage join must remain unbound")
    require(manifest.get("contains_private_key_or_license") is False, "secret-material flag must be false")
    require(manifest.get("contains_pcm_or_media") is False, "media flag must be false")
    require(manifest.get("contains_native_source_or_decompilation") is False, "native-code flag must be false")
    print(f"PASS package={PACKAGE_ID} payloads={len(expected)} config_tree=BOUND typed_storage=UNBOUND")


if __name__ == "__main__":
    if "--build" in sys.argv:
        build()
    else:
        check()
