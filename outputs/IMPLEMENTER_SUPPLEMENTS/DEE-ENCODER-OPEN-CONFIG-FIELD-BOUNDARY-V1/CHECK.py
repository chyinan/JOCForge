from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "DEE-ENCODER-OPEN-CONFIG-FIELD-BOUNDARY-V1"
SOURCE_MANIFEST = "7b30ee2e340f03a354fbc8ff5f22c39d434511326cf18366bbd16a0b10bfd551"
STATUS = "ENCODER_OPEN_CONFIG_BUILDER_BOUND; REQUEST_FIELD_SEMANTICS_PARTLY_OPAQUE"


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
    config = evidence["encoder_open_config"]
    require(config["buffer_size_bytes"] == 120, "config size")
    require(config["destination_data_rate"] == "U32 config word 10 at +0x28", "data rate destination")
    require(config["job_1024_kbps_to_enum_mapping"] == "UNBOUND", "job mapping")
    require(config["sample_rate_acmod_lfeon_wire_order"] == "UNBOUND", "format fields")
    require(evidence["effect_on_pcm_goal"]["pcm_mismatch_explained"] is False, "PCM cause boundary")
    vector = load("INPUT-ONLY-VECTOR.json")
    require(len(vector["cases"]) == 2, "case count")
    require([case["expected_config_word10_u32"] for case in vector["cases"]] == [4, 5], "input-only data rate mapping")
    require(vector["job_1024_kbps_to_enum_code"] == "UNBOUND", "vector job mapping")
    print(f"PASS package={PACKAGE_ID}")
    print(f"PASS manifest_sha256={manifest_hash}")
    print("PASS encoder-open data-rate transfer and explicit format/PCM boundaries")


if __name__ == "__main__":
    main()
