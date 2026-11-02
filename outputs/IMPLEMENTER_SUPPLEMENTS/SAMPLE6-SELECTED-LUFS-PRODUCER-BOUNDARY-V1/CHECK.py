from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "SAMPLE6-SELECTED-LUFS-PRODUCER-BOUNDARY-V1"
SOURCE_MANIFEST_DIGEST = "fd6b8b3e5ee1301787288c53962e1d45afdaaeba582d59dad6a121c44941691c"
RUST_INPUT_SHA256 = "7826dcbef37ef69642a6164c97a37bcfb3f917cea74aa0f0a30d76d0a825f1d1"
STATUS = "RUST_MEASUREMENT_AND_DIALNORM_RULE_BOUND; RUN_SCALAR_AND_UNIT_UNBOUND; DEE_U5_BOUNDED_WINDOW_ONLY"


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
    require(manifest["source_reverse_manifest_sha256"] == SOURCE_MANIFEST_DIGEST, "source manifest binding")
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
    require(evidence["rust_run"]["input_sha256"] == RUST_INPUT_SHA256, "run input identity")
    require(evidence["rust_run"]["selected_lufs_word_in_run_receipt"] is None, "Rust scalar not recorded")
    require(evidence["rust_measurement_path"]["physical_unit"] == "UNBOUND", "physical unit boundary")
    require(evidence["scope_comparison"]["same_scope_comparable"] is False, "scope distinction")
    require(evidence["dee_bounded_reference"]["full_file"] is False, "DEE window boundary")
    require(evidence["run_boundaries"]["new_dee_run"] is False, "no new DEE run")
    require(evidence["run_boundaries"]["pcm_or_payload_read"] is False, "no payload read")
    print(f"PASS package={PACKAGE_ID}")
    print(f"PASS manifest_sha256={manifest_hash}")
    print("PASS Rust current formula/receipt boundary and DEE bounded-window scope")
    print("PASS no Rust scalar fabricated; no DEE/PCM/payload/key access")
    return 0


if __name__ == "__main__":
    sys.exit(main())
