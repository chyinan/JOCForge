from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "SAMPLE6-AU-ALIGNED-DEE-CORE-PCM-COMPARISON-V1"
DEE_WINDOW_MANIFEST = "ade02cf02c9e0d7f1947148a9b188accad24900e649a4e67999c9c120fb96a49"
JOCFORGE_GATE_MANIFEST = "6d5e9a96a90a940bb9d6b7969489b7b71888988d8cc048e62125690f141943a4"
JOCFORGE_GATE_SEAL = "d79d6f1311702876023c0c4525d5ea63940de6758656c6b3ffde2ae49a9dda18"
STATUS = "AU_ALIGNED_4S_CORE_PCM_HASH_MISMATCH; JOB_CONTROL_AND_SPATIAL_PARITY_OPEN"


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
    require(manifest["jocforge_gate_manifest_sha256"] == JOCFORGE_GATE_MANIFEST, "JOCForge gate binding")
    require(manifest["jocforge_gate_package_seal_sha256"] == JOCFORGE_GATE_SEAL, "JOCForge gate seal")
    payload = {entry["file"]: entry for entry in manifest["files"]}
    actual = {path.name for path in ROOT.iterdir() if path.is_file() and path.name not in {"MANIFEST.json", "MANIFEST.sha256", "package_seal.json"}}
    require(actual == set(payload), "payload inventory")
    for name, entry in payload.items():
        path = ROOT / name
        require(path.stat().st_size == entry["bytes"], f"size {name}")
        require(sha256(path) == entry["sha256"], f"payload hash {name}")
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
    require(evidence["authorized_input"]["sha256"] == evidence["jocforge_full_run_crop"]["input_sha256"], "same input")
    require(evidence["dee_reference"]["access_units"] == 125, "DEE AU count")
    require(evidence["dee_reference"]["decoded_sample_frames"] == 192000, "DEE decoded samples")
    require(evidence["dee_reference"]["pcm_recomputed_by_wash"] is False, "DEE PCM digest source-attested")
    require(evidence["jocforge_full_run_crop"]["AU_interval"] == [2750, 2875], "Rust AU slice")
    require(evidence["jocforge_full_run_crop"]["pcm_sha256_recomputed_by_wash"] == "6d9a5de71c530e65d87aa7ef203b3900ff50beec336018c7b269ed061747f2f7", "Rust hash recomputed")
    require(evidence["comparison"]["exact_hash_match"] is False, "core hash mismatch")
    require(evidence["comparison"]["scope"].startswith("E-AC-3 5.1 core"), "core-only scope")
    require(evidence["interpretation_limits"]["atmos_spatial_parity"] == "NOT_RUN", "spatial scope")
    print(f"PASS package={PACKAGE_ID}")
    print(f"PASS manifest_sha256={manifest_hash}")
    print("PASS AU-aligned same-input 4s 5.1 core hash mismatch and source bindings")
    print("PASS no PCM persisted; Atmos spatial/full-track parity not claimed")
    return 0


if __name__ == "__main__":
    sys.exit(main())
