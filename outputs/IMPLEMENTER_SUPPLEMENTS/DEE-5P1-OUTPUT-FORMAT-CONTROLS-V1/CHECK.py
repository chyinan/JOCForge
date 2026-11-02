from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "DEE-5P1-OUTPUT-FORMAT-CONTROLS-V1"
SOURCE_MANIFEST = "05d1bc3ef9c8a0e43d8ff83d605f4fc7dc56708e3a71e208e2dcfc81ccb6f686"
STATUS = "HIGH_LEVEL_STREAM_METADATA_MATCHED; FRAME_CONTROL_TO_NATIVE_PROVIDER_UNBOUND"


def require(condition: bool, label: str) -> None:
    if not condition:
        raise SystemExit(f"FAIL {label}")


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def load(name: str) -> dict:
    return json.loads((ROOT / name).read_text(encoding="utf-8"))


def main() -> None:
    manifest = load("MANIFEST.json")
    manifest_hash = sha256(ROOT / "MANIFEST.json")
    require(manifest_hash == (ROOT / "MANIFEST.sha256").read_text(encoding="ascii").split()[0], "manifest hash")
    require(manifest["package_id"] == PACKAGE_ID and manifest["status"] == STATUS, "manifest id/status")
    require(manifest["source_reverse_manifest_sha256"] == SOURCE_MANIFEST, "source manifest")
    entries = {entry["file"]: entry for entry in manifest["files"]}
    actual = {
        path.name for path in ROOT.iterdir()
        if path.is_file() and path.name not in {"MANIFEST.json", "MANIFEST.sha256", "package_seal.json"}
    }
    require(actual == set(entries), "payload inventory")
    for name, entry in entries.items():
        path = ROOT / name
        require(path.stat().st_size == entry["bytes"], f"size {name}")
        require(sha256(path) == entry["sha256"], f"hash {name}")
    seal = load("package_seal.json")
    sealed = {entry["file"]: entry for entry in seal["files"]}
    require(seal["package_id"] == PACKAGE_ID, "seal id")
    require(set(sealed) == set(entries) | {"MANIFEST.json", "MANIFEST.sha256"}, "seal inventory")
    for name, entry in sealed.items():
        path = ROOT / name
        require(path.stat().st_size == entry["bytes"], f"sealed size {name}")
        require(sha256(path) == entry["sha256"], f"sealed hash {name}")
    evidence = load("EVIDENCE.json")
    require(evidence["status"] == STATUS, "evidence status")
    job = evidence["selected_dee_job"]
    require(job["explicit_fields"]["data_rate_kbps"] == 1024 and job["explicit_fields"]["line_mode_drc_profile"] == "none", "job controls")
    require("acmod" in job["xml_absent_fields"] and "lfeon" in job["xml_absent_fields"], "absent fields")
    require(evidence["existing_stream_metadata"]["dee"]["channel_layout"] == "5.1(side)", "DEE metadata")
    require(evidence["existing_stream_metadata"]["rust"]["channel_layout"] == "5.1(side)", "Rust metadata")
    require(evidence["native_frame_writer_boundary"]["frame_provider_format_control_producer"] == "UNBOUND", "native format boundary")
    require(evidence["pcm_mismatch_relation"]["explained_by_this_package"] is False, "PCM cause boundary")
    require(evidence["scope"]["dee_run"] == "NOT_RUN" and evidence["scope"]["pcm_or_payload_read"] is False, "scope")
    vector = load("INPUT-ONLY-VECTOR.json")
    require(vector["expected_disposition"]["container_difference"] == "packaging only", "container scope")
    require(vector["expected_disposition"]["frame_control_equivalence"] == "NOT_EVALUATED", "frame parity scope")
    print(f"PASS package={PACKAGE_ID}")
    print(f"PASS manifest_sha256={manifest_hash}")
    print("PASS explicit profile controls, metadata scope, and UNBOUND frame-format fields")


if __name__ == "__main__":
    main()
