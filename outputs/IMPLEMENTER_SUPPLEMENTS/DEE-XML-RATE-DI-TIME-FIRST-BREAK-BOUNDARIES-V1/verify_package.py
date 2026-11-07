from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent


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
    require(manifest["package"] == "DEE-XML-RATE-DI-TIME-FIRST-BREAK-BOUNDARIES-V1", "wrong package")
    require(seal["manifest_sha256"] == sha256(manifest_path), "manifest hash mismatch")
    require(seal["status"] == "SEALED_STATIC_BOUNDARIES_ONLY", "wrong seal status")
    for entry in manifest["payloads"]:
        path = ROOT / entry["path"]
        require(path.is_file(), f"missing payload: {entry['path']}")
        require(path.stat().st_size == entry["bytes"], f"size mismatch: {entry['path']}")
        require(sha256(path) == entry["sha256"], f"hash mismatch: {entry['path']}")
    findings = (ROOT / "FINDINGS.md").read_text(encoding="utf-8")
    vectors = json.loads((ROOT / "INPUT-ONLY-VECTORS.json").read_text(encoding="utf-8"))
    require("UNBOUND" in findings and "NOT_RUN" in findings, "open boundaries were not retained")
    require(vectors["runtime_status"] == "NOT_RUN", "runtime status was promoted")
    require(vectors["pcm_or_payload_persisted"] is False, "PCM/payload boundary violated")
    require(vectors["key_or_license_accessed"] is False, "secret boundary violated")
    print(json.dumps({"status": "PASS", "payloads_verified": len(manifest["payloads"]), "manifest_sha256": sha256(manifest_path)}))


if __name__ == "__main__":
    main()
