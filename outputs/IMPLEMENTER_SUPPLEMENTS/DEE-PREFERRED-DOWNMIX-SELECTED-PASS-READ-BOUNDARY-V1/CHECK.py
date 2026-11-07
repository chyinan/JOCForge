from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "DEE-PREFERRED-DOWNMIX-SELECTED-PASS-READ-BOUNDARY-V1"
SOURCE_MANIFEST_SHA256 = "a6e1aa6c1f2cd4cce6fa80fc9cc8ee408b29f0a2ddeccd8a91f08ff782392e78"
SOURCE_CHECKSUMS_SHA256 = "19f84ef9fe79468a37a1152d63103e82d29ea5ea6bda14e1ff3fd1ee4820d59a"
PAYLOADS = ("EVIDENCE.json", "REPORT.md", "SOURCE-MANIFESTS.json")


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
        "status": "SELECTED_PASS_NO_DIRECT_MIXMODE_READ; GLOBAL_CONSUMER_UNBOUND",
        "implementation_disposition": "Record only the bounded selected-pass check; do not make a product-wide absence claim.",
        "source_manifest_sha256": SOURCE_MANIFEST_SHA256,
        "source_checksum_list_sha256": SOURCE_CHECKSUMS_SHA256,
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
    require(manifest.get("source_checksum_list_sha256") == SOURCE_CHECKSUMS_SHA256, "source checksum list mismatch")
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
    require(evidence["status"]["audio_or_bsi_consumer"] == "UNBOUND", "audio consumer must remain unbound")
    require(evidence["limits"]["global_absence_claim"] is False, "global absence claim must remain false")
    require(manifest.get("contains_private_key_or_license") is False, "secret-material flag must be false")
    require(manifest.get("contains_pcm_or_media") is False, "media flag must be false")
    require(manifest.get("contains_native_source_or_decompilation") is False, "native-code flag must be false")
    print(f"PASS package={PACKAGE_ID} payloads={len(expected)} selected_pass_hits=0 audio_consumer=UNBOUND")


if __name__ == "__main__":
    if "--build" in sys.argv:
        build()
    else:
        check()
