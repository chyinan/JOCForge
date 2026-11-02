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

    evidence = json.loads((ROOT / "EVIDENCE.json").read_text(encoding="utf-8"))
    vector = json.loads((ROOT / "INPUT-ONLY-VECTOR.json").read_text(encoding="utf-8"))
    current = evidence["current_provider"]
    require(current["current_rust_window_words_sha256"] == current["native_window_words_sha256"], "current window hashes differ")
    require(current["index_21"]["native_f32_bits"] == current["index_21"]["current_rust_f32_bits"] == "3c0ea5aa", "current index-21 mismatch")
    require(evidence["historical_provider"]["status"] == "SUPERSEDED_BY_CURRENT_PROVIDER", "historical status mismatch")
    require(evidence["matched_lfe_boundary"]["adm_row_3_to_core_lfe_lane"] == "UNBOUND", "matched LFE lane overclaimed")
    require(evidence["matched_lfe_boundary"]["scratch_history_words_256_to_381_owner"] == "UNBOUND", "scratch history overclaimed")
    require(evidence["matched_lfe_boundary"]["effect_on_channel3_pcm_residual"] == "UNBOUND", "PCM causality overclaimed")
    require(vector["matched_pcm_output"] == "NOT_RUN", "matched output overclaimed")
    require(evidence["privacy"]["private_key_or_license_material_included"] is False, "privacy scope changed")
    print(f"PASS manifest={manifest_hash}")
    print("PASS current provider matches native; historical baseline is superseded")
    print("PASS matched LFE lineage and PCM causality remain open")


if __name__ == "__main__":
    main()
