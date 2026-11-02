from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "SAMPLE6-DEE-REFERENCE-CORE-PCM-COMPARISON-V1"
DEE_WINDOW_MANIFEST = "cce9188fe6e17a39d940861131202d74a3114a7aa70a26107fc987fb4fd39e32"
RUST_GATE_MANIFEST = "6d5e9a96a90a940bb9d6b7969489b7b71888988d8cc048e62125690f141943a4"
RUST_GATE_SEAL = "d79d6f1311702876023c0c4525d5ea63940de6758656c6b3ffde2ae49a9dda18"
STATUS = "PROVISIONAL_SAME_INPUT_CORE_PCM_HASH_MISMATCH; RUST_SAMPLE_ORIGIN_UNBOUND; SPATIAL_RENDER_NOT_COMPARED"


def require(condition: bool, label: str) -> None:
    if not condition:
        raise SystemExit(f"FAIL {label}")


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def load(name: str) -> dict:
    return json.loads((ROOT / name).read_text(encoding="utf-8"))


def main() -> int:
    manifest = load("MANIFEST.json")
    manifest_hash = sha256(ROOT / "MANIFEST.json")
    require(manifest_hash == (ROOT / "MANIFEST.sha256").read_text(encoding="ascii").split()[0], "manifest hash")
    require(manifest["package_id"] == PACKAGE_ID, "package id")
    require(manifest["status"] == STATUS, "status")
    require(manifest["dee_window_manifest_sha256"] == DEE_WINDOW_MANIFEST, "DEE window binding")
    require(manifest["jocforge_gate_manifest_sha256"] == RUST_GATE_MANIFEST, "JOCForge gate binding")
    require(manifest["jocforge_gate_package_seal_sha256"] == RUST_GATE_SEAL, "JOCForge gate seal")
    payload = {entry["file"]: entry for entry in manifest["files"]}
    actual = {path.name for path in ROOT.iterdir() if path.is_file() and path.name not in {"MANIFEST.json", "MANIFEST.sha256", "package_seal.json"}}
    require(actual == set(payload), "payload inventory")
    for name, entry in payload.items():
        path = ROOT / name
        require(path.stat().st_size == entry["bytes"], f"size {name}")
        require(sha256(path) == entry["sha256"], f"hash {name}")
    seal = load("package_seal.json")
    sealed = {entry["file"]: entry for entry in seal["files"]}
    require(seal["package_id"] == PACKAGE_ID and seal["file_count"] == len(sealed), "seal identity")
    require(set(sealed) == set(payload) | {"MANIFEST.json", "MANIFEST.sha256"}, "seal inventory")
    for name, entry in sealed.items():
        path = ROOT / name
        require(path.stat().st_size == entry["bytes"], f"sealed size {name}")
        require(sha256(path) == entry["sha256"], f"sealed hash {name}")

    evidence = load("EVIDENCE.json")
    require(evidence["status"] == STATUS, "evidence status")
    require(evidence["authorized_input"]["fixture_sha256"] == evidence["jocforge_full_run"]["input_sha256"], "same source hash")
    compare = evidence["candidate_same_window_comparison"]
    require(compare["exact_pcm_hash_match"] is False, "observed candidate hash mismatch")
    require(compare["alignment_assumption"].endswith("not proven by run receipt"), "alignment caveat")
    require(evidence["scope_limits"]["atmos_object_or_spatial_render_comparison"] == "NOT_PERFORMED", "core-only scope")
    require(evidence["scope_limits"]["private_hmac_or_license_behavior"] == "NOT_ASSESSED", "private behavior scope")
    require(evidence["exclusions"]["pcm_payload_included"] is False, "PCM exclusion")
    print(f"PASS package={PACKAGE_ID}")
    print(f"PASS manifest_sha256={manifest_hash}")
    print("PASS same-input decode-only candidate mismatch with alignment caveat")
    print("PASS no PCM persisted; core-only result does not claim Atmos spatial parity")
    return 0


if __name__ == "__main__":
    sys.exit(main())
