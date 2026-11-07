from __future__ import annotations

import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PAYLOADS = ("FINDINGS.md", "INPUT-ONLY-VECTORS.json", "SOURCE-RECEIPTS.json", "verify_package.py")

def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()

def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)

def main() -> None:
    manifest_path = ROOT / "MANIFEST.json"
    seal_path = ROOT / "package_seal.json"
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    seal = json.loads(seal_path.read_text(encoding="utf-8"))
    require(manifest.get("package") == "DEE-ADM-5-1-SURROUND-TRIM-FIRST-BREAK-V1", "wrong package")
    require(seal.get("manifest_sha256") == sha256(manifest_path), "manifest hash mismatch")
    require(seal.get("status") == "SEALED_FIRST_BREAK_ONLY", "wrong seal status")
    require(tuple(x["path"] for x in manifest["payloads"]) == PAYLOADS, "payload inventory mismatch")
    for item in manifest["payloads"]:
        path = ROOT / item["path"]
        require(path.is_file() and path.stat().st_size == item["bytes"], f"missing/size mismatch: {item['path']}")
        require(sha256(path) == item["sha256"], f"hash mismatch: {item['path']}")
    vectors = json.loads((ROOT / "INPUT-ONLY-VECTORS.json").read_text(encoding="utf-8"))
    require(vectors["runtime_status"] == "NOT_RUN", "runtime status promoted")
    require(vectors["vectors"][0]["status"] == "PROPOSED_NOT_RUN", "vector status promoted")
    require(vectors["pcm_or_payload_persisted"] is False, "PCM/payload boundary failed")
    require(vectors["key_or_license_accessed"] is False, "secret boundary failed")
    print(json.dumps({"status": "PASS", "payloads_verified": len(manifest["payloads"]), "manifest_sha256": sha256(manifest_path)}))

if __name__ == "__main__":
    main()
