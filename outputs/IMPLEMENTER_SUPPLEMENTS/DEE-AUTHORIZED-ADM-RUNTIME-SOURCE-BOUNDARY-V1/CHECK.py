from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "DEE-AUTHORIZED-ADM-RUNTIME-SOURCE-BOUNDARY-V1"
SOURCE_MANIFEST_SHA256 = "4ef1fe0034d5f6a3f6efdc98be43261d86abbdd60d6a899c6fad504a431cf655"
JOCFORGE_GATE_MANIFEST_SHA256 = "6d5e9a96a90a940bb9d6b7969489b7b71888988d8cc048e62125690f141943a4"
JOCFORGE_GATE_SEAL_SHA256 = "d79d6f1311702876023c0c4525d5ea63940de6758656c6b3ffde2ae49a9dda18"
JOCFORGE_RUN_RECEIPT_SHA256 = "ecc5a1c01d8d14b2f7a43bfd6cb2dfd03269254a98699669f9c519e9c670d766"
STATUS = "AUTHORIZED_ADM_RUNTIME_INPUT_EDGE_BOUND; PARTIAL_DEE_REFERENCE_BOUND; SEGMENT_ALIGNMENT_AND_AUDIO_PARITY_UNBOUND"


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
    require(manifest["source_reverse_manifest_sha256"] == SOURCE_MANIFEST_SHA256, "source manifest binding")
    require(manifest["jocforge_gate_manifest_sha256"] == JOCFORGE_GATE_MANIFEST_SHA256, "JOCForge gate manifest binding")
    require(manifest["jocforge_gate_package_seal_sha256"] == JOCFORGE_GATE_SEAL_SHA256, "JOCForge gate seal binding")
    payload = {entry["file"]: entry for entry in manifest["files"]}
    actual = {path.name for path in ROOT.iterdir() if path.is_file() and path.name not in {"MANIFEST.json", "MANIFEST.sha256", "package_seal.json"}}
    require(actual == set(payload), "payload inventory")
    for name, entry in payload.items():
        path = ROOT / name
        require(path.stat().st_size == entry["bytes"], f"payload size {name}")
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
    require(evidence["authorized_fixture"]["sha256"] == evidence["jocforge_full_run_identity"]["input_sha256"], "same input hash")
    rust = evidence["jocforge_full_run_identity"]
    require(rust["run_receipt_sha256"] == JOCFORGE_RUN_RECEIPT_SHA256, "JOCForge run receipt binding")
    sources = {row["package_id"]: row for row in evidence["source_packages"]}
    require(sources["SAMPLE6-KEYLESS-DEE-REFERENCE-GATE-V4"]["manifest_sha256"] == JOCFORGE_GATE_MANIFEST_SHA256, "JOCForge gate manifest binding")
    require(sources["SAMPLE6-KEYLESS-DEE-REFERENCE-GATE-V4"]["package_seal_sha256"] == JOCFORGE_GATE_SEAL_SHA256, "JOCForge gate seal binding")
    require(evidence["dee_reference"]["time_base"] == "file_position", "bounded window timebase")
    require(evidence["dee_reference"]["output_probe"]["duration_seconds"] == 2.048, "reference duration")
    require(evidence["runtime_input_edge"]["same_output_wrapper_identity_joined"] is True, "runtime wrapper identity")
    require(evidence["runtime_input_edge"]["path_handler_builds_source_registry"] is False, "source registry boundary")
    require(evidence["runtime_input_edge"]["typed_origin_of_source_containers"] == "UNBOUND", "source container origin")
    require(evidence["wash_observation"]["decoded_pcm_compared"] is False, "PCM comparison boundary")
    require(evidence["wash_observation"]["segment_alignment"] == "UNBOUND", "segment mapping boundary")
    print(f"PASS package={PACKAGE_ID}")
    print(f"PASS manifest_sha256={manifest_hash}")
    print("PASS same ADM file identity, bounded DEE reference, and runtime wrapper join")
    print("PASS segment/audio parity and source-container origin remain UNBOUND; no media payload/key")
    return 0


if __name__ == "__main__":
    sys.exit(main())
