from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "SAMPLE6-DIALNORM-DRC-CORE-PCM-BOUNDARY-V1"
SOURCE_MANIFEST = "21e28adfd28873ada29a035096c6493d30df76ff9db76203110fa06d840a7cee"
STATUS = "DIALNORM_TYPED_FRAME_PREP_CLOSED; FINAL_BSI_AND_CORE_PCM_EFFECT_OPEN; DRC_PATH_UNBOUND"


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
    dialnorm = evidence["dialnorm"]
    require(dialnorm["parameter_id"] == "0x12", "parameter id")
    require(dialnorm["typed_path"] == ["per_stream+0x9c dword", "nested_context+0x34 dword", "nested_context+0x19140 word", "frame-preparation branch", "per-frame record at frame_set+index*0x38+0xd0"], "typed path")
    require(dialnorm["final_bsi_u5_writer_join"] == "OPEN", "BSI boundary")
    require(dialnorm["core_pcm_or_coefficient_consumer_join"] == "NOT_FOUND_IN_BOUNDED_TRACE", "PCM boundary")
    require(evidence["drc"]["drc_specific_field_to_pcm_or_bsi_path"] == "UNBOUND", "DRC boundary")
    require(evidence["drc"]["adjacent_stream_plus_a8_field"] == "UNBOUND_SEMANTICS; DO_NOT_LABEL_AS_DRC", "adjacent field")
    vector = load("INPUT-ONLY-VECTOR.json")
    require(vector["kind"] == "illustrative_not_measured", "vector scope")
    require(vector["hypothetical_preencode_gain"]["not_a_dee_drc_claim"] is True, "gain caveat")
    require(evidence["audio_effect_relation"]["effect_on_4s_core_pcm_mismatch"] == "NOT_ESTABLISHED", "PCM attribution")
    print(f"PASS package={PACKAGE_ID}")
    print(f"PASS manifest_sha256={manifest_hash}")
    print("PASS DialNorm frame-prep path and BSI/PCM/DRC boundaries")


if __name__ == "__main__":
    main()
