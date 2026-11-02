from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "ADM-CONVERTER-ROW-TABLE-PRODUCER-BOUNDARY-V1"
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
    require(evidence["target_image"]["sha256"] == "9972be4f952ca5f9121fa83ef941b29d4c1e6baf0ff1f708baaaedffa1bd76a8", "target image mismatch")
    require(evidence["static_rows"]["caller_row_order_preserved"] is True, "row order behavior mismatch")
    require(evidence["row_identity"]["descriptor_to_physical_row_pointer"] == "UNBOUND", "physical row join promoted")
    require(evidence["input_identity"]["pcm_values_decoded_or_exported"] is False, "PCM boundary mismatch")
    require(evidence["scope"]["private_key_hmac_or_license_read"] is False, "secret boundary mismatch")
    print("PACKAGE_VERIFIED files=4 static_row_order=true chna_physical_join_unbound=true")


if __name__ == "__main__":
    main()
