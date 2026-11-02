from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "F13C-DEE-OMITTED-DOWNMIX-VS-RUST-CONFIG-V1"
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
    dee = evidence["observed_dee_dme_behavior"]
    rust = evidence["observed_rust_behavior"]
    require(dee["preferred_downmix_mode_emitted"] is False, "DEE omission changed")
    require(dee["effective_runtime_default"] == "UNBOUND", "native default promoted")
    require(rust["generated_job_xml"]["preferred_downmix_mode"] == "loro", "Rust XML snapshot changed")
    require(evidence["comparison"]["native_effective_value_mismatch_proven"] is False, "native mismatch overclaimed")
    require(evidence["scope"]["private_key_hmac_or_license_read"] is False, "secret boundary mismatch")
    print("PACKAGE_VERIFIED files=4 config_omission_bound=true native_default_unbound=true")


if __name__ == "__main__":
    main()
