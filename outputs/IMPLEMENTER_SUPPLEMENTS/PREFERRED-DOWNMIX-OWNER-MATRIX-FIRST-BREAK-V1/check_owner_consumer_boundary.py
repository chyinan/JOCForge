#!/usr/bin/env python3
"""Validate the Wash-only preferred-downmix owner/consumer evidence package."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path


PACKAGE = Path(__file__).resolve().parent


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(f"downmix boundary package check failed: {message}")


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def read_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def main() -> None:
    manifest_path = PACKAGE / "MANIFEST.json"
    manifest = read_json(manifest_path)
    seal = (PACKAGE / "MANIFEST.sha256").read_text(encoding="ascii").split()[0]
    require(sha256(manifest_path) == seal, "manifest seal mismatch")
    for entry in manifest["artifacts"]:
        path = PACKAGE / entry["path"]
        require(path.is_file(), f"missing local artifact: {entry['path']}")
        require(path.stat().st_size == entry["bytes"], f"local artifact size mismatch: {entry['path']}")
        require(sha256(path) == entry["sha256"], f"local artifact hash mismatch: {entry['path']}")

    identities = read_json(PACKAGE / "SOURCE-IDENTITIES.json")
    for package in identities["packages"]:
        root = Path(package["root"])
        package_manifest = root / "MANIFEST.json"
        require(package_manifest.is_file(), f"missing reused manifest: {package['package']}")
        require(sha256(package_manifest) == package["manifest_sha256"], f"reused manifest hash mismatch: {package['package']}")
        recorded = read_json(package_manifest)
        payloads = recorded.get("payloads") or recorded.get("files") or recorded.get("artifacts") or []
        for payload in package["payloads"]:
            source = root / payload["path"]
            require(source.is_file(), f"missing reused payload: {package['package']}/{payload['path']}")
            expected = next((item.get("sha256") for item in payloads if item.get("path") == payload["path"]), None)
            require(expected == payload["sha256"], f"payload identity not in manifest: {package['package']}/{payload['path']}")
            require(sha256(source) == payload["sha256"], f"reused payload hash mismatch: {package['package']}/{payload['path']}")

    boundary = read_json(PACKAGE / "OWNER-CONSUMER-BOUNDARY.json")
    mode = boundary["selected_direct_edge"]
    require(mode["caller"] == "FUN_1801475f0" and mode["child"] == "FUN_1801535b0", "selected direct edge drift")
    require(mode["caller_instruction_hits_for_priv_plus_0x980"] == 0, "caller field read state changed")
    require(mode["child_instruction_hits_for_priv_plus_0x980"] == 0, "child field read state changed")
    require(mode["first_actual_channel_matrix_or_pcm_consumer"] == "UNBOUND", "consumer boundary was promoted")
    require(boundary["classification"]["bsi_or_signaling_only"] == "UNBOUND", "signaling classification was promoted")
    require(boundary["classification"]["sample_transform"] == "UNBOUND", "sample-transform classification was promoted")
    require(boundary["classification"]["runtime"] == "NOT_RUN", "runtime status changed")
    require(boundary["access_boundary"]["new_dee_run"] is False, "DEE execution boundary changed")
    require(boundary["access_boundary"]["media_pcm_payload_key_license_read"] is False, "protected material boundary changed")
    require(identities["audit_boundary"]["implementation_contact"] is False, "implementation boundary changed")
    print("PASS: local seal, reused manifest/payload hashes, one-hop owner edge, and UNBOUND consumer boundary")


if __name__ == "__main__":
    main()
