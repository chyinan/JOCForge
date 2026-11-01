from __future__ import annotations

import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent

def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)

def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()

def main() -> None:
    manifest_bytes = (ROOT / "MANIFEST.json").read_bytes()
    manifest = json.loads(manifest_bytes)
    require(manifest.get("package_id") == "pcm24_ddp_reader_adapter_lineage_run151_v1", "package id mismatch")
    require(manifest.get("implementation_delivery") == "ALLOWED_WITHIN_DECLARED_SCOPE_ONLY", "scope mismatch")
    require(manifest.get("status", "").endswith("OUTPUT_PARSE_PLAYBACK_NOT_RUN"), "status overclaim")
    source = json.loads((ROOT / "SOURCE-BINDING.json").read_text(encoding="utf-8"))
    require(source.get("incoming_research_manifest_sha256") == "ae7c3e737eba3fbd1ed5dc70daebc3dff546b4cf8042dbd30883a556c8858389", "source manifest mismatch")
    require(source.get("input_rehashed_for_run151") is False and source.get("input_identity_status") == "UNBOUND/NO", "input identity overstated")
    evidence = json.loads((ROOT / "EVIDENCE.json").read_text(encoding="utf-8"))
    require(evidence.get("run_scope", {}).get("input_identity_status") == "UNBOUND/NO", "run input identity overstated")
    require(evidence.get("ddp_source_record_to_reader", {}).get("row_contents_captured") is False, "row contents scope changed")
    require(evidence.get("output_context", {}).get("bitstream_parse") == "NOT_RUN", "bitstream status overclaimed")
    require(evidence.get("output_context", {}).get("playback") == "NOT_RUN", "playback status overclaimed")
    for entry in manifest.get("payloads", []):
        name = entry.get("name")
        require(isinstance(name, str), "payload name missing")
        path = (ROOT / name).resolve()
        require(path.parent == ROOT.resolve(), f"payload escaped package: {name}")
        require(path.is_file(), f"missing payload: {name}")
        require(path.stat().st_size == entry.get("bytes"), f"size mismatch: {name}")
        require(sha256(path) == entry.get("sha256"), f"hash mismatch: {name}")
    sidecar = (ROOT / "MANIFEST.sha256").read_text(encoding="ascii").split()
    require(sidecar and sidecar[0] == hashlib.sha256(manifest_bytes).hexdigest(), "manifest sidecar mismatch")
    verification = json.loads((ROOT / "VERIFICATION.json").read_text(encoding="utf-8"))
    require(verification.get("package_integrity") in {"PENDING_PACKAGE_CHECK", "PASS_PACKAGE_INTEGRITY_ONLY"}, "verification scope mismatch")
    print("PASS: scope, source binding, hashes, UNBOUND, and NOT_RUN verified")

if __name__ == "__main__":
    main()
