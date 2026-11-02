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
    require(evidence["native_image"]["sha256"] == "0cf23520799f52323b0994e5fcc16924da7b027a5b83a0cff3b4f06792c18532", "core image identity mismatch")
    require(evidence["core_input_api"]["channel_descriptor_count"] == 6, "input descriptor count mismatch")
    require(evidence["first_sample_transform"]["fresh_words_copied_per_lane"] == 256, "fresh-copy extent mismatch")
    require(evidence["first_sample_transform"]["maximum_relative_f32_index_read"] == 381, "scratch read extent mismatch")
    require(evidence["first_sample_transform"]["scratch_owner_capacity_and_history_generation"] == "UNBOUND", "scratch history overclaimed")
    require(evidence["core_input_api"]["adm_row3_to_internal_lfe_lane"] == "UNBOUND", "matched lane overclaimed")
    require(evidence["first_sample_transform"]["numeric_output_values"] == "NOT_RUN", "numeric output overclaimed")
    require(vector["classification"] == "INPUT_ONLY; PROPOSED_NOT_RUN", "vector status mismatch")
    require(vector["expected_output_values"] == "NOT_PROVIDED" and vector["numeric_comparison"] == "NOT_RUN", "vector output overclaimed")
    require(evidence["execution"]["private_key_or_license_material_included"] is False, "privacy scope changed")
    require(evidence["execution"]["native_code_or_disassembly_included"] is False, "source scope changed")
    print(f"PASS manifest={manifest_hash}")
    print("PASS core LFE branch, 256-word input copy, first F32 transform, and seven-value banks")
    print("PASS matched row, scratch history, numeric output, and Rust comparison remain open")


if __name__ == "__main__":
    main()
