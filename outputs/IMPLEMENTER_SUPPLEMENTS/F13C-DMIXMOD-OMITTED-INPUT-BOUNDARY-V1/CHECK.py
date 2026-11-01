from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "F13C-DMIXMOD-OMITTED-INPUT-BOUNDARY-V1"
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
    require(evidence["source"]["manifest_sha256"] == "cfaec056f570eec6d2a1d985bd3544df30eac446f89140ace9fa551448703415", "source manifest mismatch")
    require(evidence["front_end_input"]["PreferredStereoDownmix_active"] is False, "inactive UI state changed")
    require(evidence["front_end_input"]["inactive_request_behavior"] == "preferred_downmix_mode entry omitted", "omission boundary lost")
    require(evidence["native_parser"]["omitted_field_effective_runtime_default"] == "UNBOUND", "native default promoted")
    require(evidence["implementation_disposition"]["do_not_ask_user_when_control_inactive"] is True, "user prompt boundary lost")
    require(evidence["execution_scope"]["private_key_hmac_or_license_read"] is False, "secret boundary mismatch")
    print("PACKAGE_VERIFIED files=4 omitted_input_bound=true native_default_unbound=true")


if __name__ == "__main__":
    main()
