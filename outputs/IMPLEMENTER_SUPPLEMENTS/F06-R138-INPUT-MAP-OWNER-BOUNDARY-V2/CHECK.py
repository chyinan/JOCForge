from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "F06-R138-INPUT-MAP-OWNER-BOUNDARY-V2"
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
    state = evidence["process_state"]
    require(state["descriptor_context_plus_0x138_to_state_plus_0x08_vector"] == "BOUND_STATIC_REGISTER_FLOW", "typed context edge mismatch")
    require(state["context_vector_plus_0x08_to_input_map_plus_0xa0"] == "UNBOUND", "input map owner promoted")
    require(state["input_metadata_map_plus_0xa0"]["domain"] == "WAV parseWav", "input map domain mismatch")
    require(state["final_plus_0xfc8_value"] == "UNBOUND", "runtime value promoted")
    print("PACKAGE_VERIFIED files=4 descriptor_context_bound_input_map_owner_unbound=true")


if __name__ == "__main__":
    main()
