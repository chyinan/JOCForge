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
    table = evidence["window_table"]
    require(table["entry_count"] == 256 and table["mismatch_count"] == 12, "window mismatch count")
    require(table["first_mismatch"] == {"index": 21, "native_f32_bits": "3c0ea5aa", "rust_f32_bits": "3c0ea5a9"}, "first mismatch identity")
    require(table["long_cosine_word_mismatches"] == 0 and table["long_sine_word_mismatches"] == 0 and table["spectrum_permutation_mismatches"] == 0, "matched companion tables mismatch")
    require(evidence["matched_lfe_boundary"]["layout_selector_and_selected_row"] == "UNBOUND", "matched layout overclaimed")
    require(evidence["matched_lfe_boundary"]["scratch_history_256_to_381_owner"] == "UNBOUND", "scratch owner overclaimed")
    require(evidence["matched_lfe_boundary"]["effect_on_channel3_pcm_residual"] == "UNBOUND", "PCM causality overclaimed")
    require(vector["execution"] == "NOT_RUN" and vector["expected_pcm_output"] == "NOT_PROVIDED", "vector execution overclaimed")
    require(evidence["privacy"]["private_key_or_license_material_included"] is False, "privacy scope changed")
    require(evidence["privacy"]["native_code_or_disassembly_included"] is False, "source scope changed")
    print(f"PASS manifest={manifest_hash}")
    print("PASS 12 one-ULP window split and matched companion tables")
    print("PASS matched LFE lane, scratch history, and PCM causality remain open")


if __name__ == "__main__":
    main()
