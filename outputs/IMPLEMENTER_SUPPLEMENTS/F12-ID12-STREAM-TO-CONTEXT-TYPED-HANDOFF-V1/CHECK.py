from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "F12-ID12-STREAM-TO-CONTEXT-TYPED-HANDOFF-V1"
PAYLOADS = (
    "EVIDENCE.json",
    "INPUT-ONLY-VECTORS.json",
    "REPORT.md",
    "SOURCE-MANIFESTS.json",
)


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def main() -> None:
    manifest_path = ROOT / "MANIFEST.json"
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    pinned = (ROOT / "MANIFEST.sha256").read_text(encoding="ascii").split()[0]
    require(sha256(manifest_path) == pinned, "manifest hash mismatch")
    require(manifest["schema"] == "jocforge-washed-implementer-supplement.v1", "schema mismatch")
    require(manifest["package_id"] == PACKAGE_ID, "package id mismatch")
    require(manifest["implementation_ready"] is False, "scope must remain non-ready")
    require(tuple(item["file"] for item in manifest["files"]) == PAYLOADS, "payload list mismatch")
    for item in manifest["files"]:
        path = ROOT / item["file"]
        require(path.is_file(), f"missing payload: {item['file']}")
        require(path.stat().st_size == item["bytes"], f"size mismatch: {item['file']}")
        require(sha256(path) == item["sha256"], f"payload hash mismatch: {item['file']}")
    evidence = json.loads((ROOT / "EVIDENCE.json").read_text(encoding="utf-8"))
    handoff = evidence["typed_handoff"]
    require(handoff["corrected_pointer_arithmetic"] == "param_2 is longlong*; param_2+0x0c = stream+0x60", "pointer correction lost")
    require(handoff["source_read"] == "FUN_1403471f0 reads [RCX+0x3c] = stream+0x9c", "source field mismatch")
    require(evidence["status"]["context_0x19140_to_final_u5_bsi_write"] == "UNBOUND_TO_R5_SELECTOR", "U5 boundary promoted")
    require(evidence["typed_handoff"]["runtime_generation_token"] == "NOT_CAPTURED", "generation boundary promoted")
    require(evidence["status"]["media_payload_read"] is False, "media boundary mismatch")
    print("PACKAGE_VERIFIED files=4 static_typed_handoff=true final_u5_unbound=true")


if __name__ == "__main__":
    main()
