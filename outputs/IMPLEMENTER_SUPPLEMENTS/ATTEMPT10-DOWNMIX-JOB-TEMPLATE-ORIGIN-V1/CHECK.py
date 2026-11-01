from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "ATTEMPT10-DOWNMIX-JOB-TEMPLATE-ORIGIN-V1"
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
    require(evidence["source"]["manifest_sha256"] == "46902ddb8b0ef5db60586bccc7405d301daf904ef49e0974af48eec383957a49", "source manifest mismatch")
    require(evidence["rust_template_source"]["downmix_fields_are_literal_template_values"] is True, "template origin lost")
    require(evidence["attempt10_xml_reconstruction"]["captured_by_application_at_runtime"] is False, "runtime XML promoted")
    require(evidence["dee_ui_distinction"]["effective_runtime_default"] == "UNBOUND" if "effective_runtime_default" in evidence["dee_ui_distinction"] else evidence["dee_ui_distinction"]["native_runtime_default_when_field_absent"] == "UNBOUND", "DEE default promoted")
    require(evidence["execution_scope"]["private_key_hmac_or_license_read"] is False, "secret boundary mismatch")
    print("PACKAGE_VERIFIED files=4 source_template_bound_runtime_xml_reconstructed=false")


if __name__ == "__main__":
    main()
