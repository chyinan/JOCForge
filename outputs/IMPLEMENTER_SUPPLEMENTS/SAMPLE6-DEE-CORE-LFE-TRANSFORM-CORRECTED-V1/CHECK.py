from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PAYLOADS = {"REPORT.md", "EVIDENCE.json", "INPUT-ONLY-VECTOR.json", "SOURCE-MANIFESTS.json", "CHECK.py"}


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(f"FAIL: {message}")


def digest(path: Path) -> str:
    hasher = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1 << 20), b""):
            hasher.update(chunk)
    return hasher.hexdigest()


def main() -> None:
    manifest_path = ROOT / "MANIFEST.json"
    sums_path = ROOT / "SHA256SUMS"
    seal_path = ROOT / "package_seal.json"
    manifest_hash = digest(manifest_path)
    require((ROOT / "MANIFEST.sha256").read_text(encoding="utf-8").split() == [manifest_hash, "MANIFEST.json"], "manifest sidecar mismatch")
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    files = manifest.get("files")
    require(isinstance(files, list), "manifest files missing")
    names = {entry.get("name") for entry in files}
    require(names == PAYLOADS, "payload set mismatch")
    for entry in files:
        path = ROOT / entry["name"]
        require(path.is_file(), f"missing {entry['name']}")
        require(path.stat().st_size == entry.get("bytes"), f"size mismatch: {entry['name']}")
        require(digest(path) == entry.get("sha256"), f"hash mismatch: {entry['name']}")
    sums = {}
    for line in sums_path.read_text(encoding="utf-8").splitlines():
        if line.strip():
            file_hash, name = line.split("  ", 1)
            sums[name] = file_hash
    require(set(sums) == PAYLOADS | {"MANIFEST.json"}, "SHA256SUMS scope mismatch")
    for name, file_hash in sums.items():
        require(digest(ROOT / name) == file_hash, f"SHA256SUMS mismatch: {name}")
    seal = json.loads(seal_path.read_text(encoding="utf-8"))
    require(seal.get("manifest_sha256") == manifest_hash, "seal manifest mismatch")
    require(seal.get("sha256sums_sha256") == digest(sums_path), "seal sums mismatch")
    require(seal.get("payload_count") == len(PAYLOADS), "seal payload count mismatch")

    evidence = json.loads((ROOT / "EVIDENCE.json").read_text(encoding="utf-8"))
    vector = json.loads((ROOT / "INPUT-ONLY-VECTOR.json").read_text(encoding="utf-8"))
    scratch = evidence["input_scratch"]
    require(scratch["fresh_copy_f32_words"] == 256, "input copy extent mismatch")
    require(scratch["input_read_index_range_inclusive"] == [0, 255], "input extent mismatch")
    require(scratch["inline_capacity_bytes"] == 1024 and scratch["allocation_request_bytes"] == 1056, "scratch sizing mismatch")
    require(evidence["separate_transform_state"]["persistent_per_slot_state"] == "state+0x266f0 + selected_map_slot*0x400", "persistent state owner mismatch")
    require(evidence["separate_temporary"]["role"].startswith("transform param_3 temporary"), "temporary owner mismatch")
    require(evidence["core_input_api"]["matched_layout_id"] == "UNBOUND", "layout status overclaimed")
    require(evidence["core_input_api"]["adm_row3_to_internal_lfe_lane"] == "UNBOUND", "row3 lane overclaimed")
    require(evidence["matched_job_xml"]["filter"] == "encode_to_atmos_ddp", "matched filter identity mismatch")
    require(evidence["matched_job_xml"]["sample_rate_or_channel_layout_fields"] == "NOT_PRESENT_IN_XML", "XML scope overclaimed")
    require(evidence["matched_job_xml"]["core_mode_or_layout_selector_value"] == "UNBOUND", "matched selector overclaimed")
    require(evidence["lfe_continuation"]["numeric_comparison"] == "NOT_RUN", "numeric comparison overclaimed")
    require(vector["numeric_comparison"] == "NOT_RUN" and vector["expected_output"] == "NOT_PROVIDED", "vector result overclaimed")
    require(evidence["execution"]["private_key_or_license_material_included"] is False, "privacy scope changed")
    print(f"PASS manifest={manifest_hash}")
    print("PASS corrected 256-word input extent and separate history/temporary ownership")
    print("PASS matched layout, row-3 lane, and numeric comparison remain open")


if __name__ == "__main__":
    main()
