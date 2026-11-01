from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "ADM-CONVERTER-STATIC-READER-SAMPLE-PATH-V1"
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
    manifest_bytes = manifest_path.read_bytes()
    manifest = json.loads(manifest_bytes)
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
    require(evidence["status"]["physical_bwf_row_table_producer_and_order"] == "UNBOUND", "row boundary lost")
    require(evidence["status"]["runtime_sample_callback_target"] == "UNBOUND", "runtime boundary lost")
    require(evidence["status"]["actual_read_count_and_eof"] == "NOT_RUN", "run status promoted")
    require(evidence["execution_scope"]["private_key_or_hmac_read"] is False, "secret boundary mismatch")
    vectors = json.loads((ROOT / "INPUT-ONLY-VECTORS.json").read_text(encoding="utf-8"))
    require(vectors["vectors"][0]["derived_arithmetic"]["adjusted_row_pointer_values"] == [4108, 8204], "row vector mismatch")
    require(vectors["vectors"][1]["derived_arithmetic"]["absolute_file_offset"] == 4224, "offset vector mismatch")
    print("PACKAGE_VERIFIED files=4 static_only=true implementation_ready=false")


if __name__ == "__main__":
    main()
