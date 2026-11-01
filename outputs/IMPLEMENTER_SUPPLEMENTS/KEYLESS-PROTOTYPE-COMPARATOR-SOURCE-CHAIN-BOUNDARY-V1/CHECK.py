from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "KEYLESS-PROTOTYPE-COMPARATOR-SOURCE-CHAIN-BOUNDARY-V1"
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
    require(evidence["source"]["source_manifest_sha256"] == "c80f5b3aefd79f990804eb3cf711b42911207e0b24c57bf35415752e6ff97853", "source manifest mismatch")
    require(evidence["user_designated_comparator"]["python_run_to_input"] == "UNBOUND", "run binding promoted")
    require(evidence["source_chain"]["preencoded_model_source_stream"]["adm_to_stream_lineage"] == "UNBOUND", "ADM lineage promoted")
    require(evidence["source_chain"]["external_helpers"]["regenerate_crc2_source_available"] is False, "helper availability changed")
    require(evidence["source_level_differences"]["audio_essence_equivalence"] == "NOT_MEASURED", "audio equivalence promoted")
    require(evidence["scope"]["private_key_hmac_or_license_read"] is False, "secret boundary mismatch")
    print("PACKAGE_VERIFIED files=4 source_only=true implementation_ready=false")


if __name__ == "__main__":
    main()
