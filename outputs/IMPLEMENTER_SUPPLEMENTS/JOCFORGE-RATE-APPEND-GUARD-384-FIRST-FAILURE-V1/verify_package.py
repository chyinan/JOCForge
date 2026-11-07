from __future__ import annotations

import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PAYLOADS = ("FINDINGS.md", "INPUT-ONLY-VECTORS.json", "SOURCE-IDENTITIES.json", "verify_package.py")

def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()

def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)

def main() -> None:
    manifest_path = ROOT / "MANIFEST.json"
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    seal = json.loads((ROOT / "package_seal.json").read_text(encoding="utf-8"))
    require(manifest.get("package") == "JOCFORGE-RATE-APPEND-GUARD-384-FIRST-FAILURE-V1", "wrong package")
    require(seal.get("manifest_sha256") == sha256(manifest_path), "manifest hash mismatch")
    require(seal.get("status") == "SEALED_IMPLEMENTATION_FAILURE_DIAGNOSIS", "wrong seal status")
    require(tuple(item["path"] for item in manifest["payloads"]) == PAYLOADS, "payload inventory mismatch")
    for item in manifest["payloads"]:
        path = ROOT / item["path"]
        require(path.is_file() and path.stat().st_size == item["bytes"], f"missing/size mismatch: {item['path']}")
        require(sha256(path) == item["sha256"], f"hash mismatch: {item['path']}")
    vectors = json.loads((ROOT / "INPUT-ONLY-VECTORS.json").read_text(encoding="utf-8"))
    require(vectors["selected_frame_bytes"] == 1536, "wrong selected frame bytes")
    require(vectors["static_guard"]["hardcoded_expected_bytes"] == 4096, "missing observed guard")
    require(vectors["static_guard"]["predicate_fails_for_selected_profile"] is True, "failure not derived")
    require(vectors["raw_partial_content_read"] is False and vectors["pcm_persisted"] is False, "audio access boundary failed")
    print(json.dumps({"status": "PASS", "payloads_verified": len(manifest["payloads"]), "manifest_sha256": sha256(manifest_path)}))

if __name__ == "__main__":
    main()
