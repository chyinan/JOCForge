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
    current = evidence["wash_direct_source_check"]
    require(current["source_file_sha256"] == "963a176760f7c7d43dba138553717b7d5273a9457a574f658ee73eeb62e7da5c", "source file hash mismatch")
    require(current["window_words_sha256"] == "097ea95140024fd143ed18a78299fa72d69bce1c8e0d356aed2efff2a6a099a2", "source words hash mismatch")
    require(current["window_words_sha256"] == current["native_window_words_sha256"], "current/native window mismatch")
    require(current["index_21_f32_bits"]["native"] == current["index_21_f32_bits"]["rust"] == "3c0ea5aa", "index 21 mismatch")
    require(evidence["historical_provider"]["status"] == "SUPERSEDED; NOT_CURRENT_DIFFERENCE", "historical status mismatch")
    require(evidence["matched_lfe_boundary"]["adm_row_3_to_core_lfe_lane"] == "UNBOUND", "matched lane overclaimed")
    require(evidence["matched_lfe_boundary"]["scratch_history_words_256_to_381_owner"] == "UNBOUND", "scratch history overclaimed")
    require(vector["pcm_output"] == "NOT_RUN" and vector["matched_lfe_lane"] == "UNBOUND", "matched output status changed")
    require(evidence["privacy"]["rust_source_code_included"] is False, "source inclusion scope changed")
    require(evidence["privacy"]["private_key_or_license_material_included"] is False, "privacy scope changed")
    print(f"PASS manifest={manifest_hash}")
    print("PASS Wash direct provider hash and current/native table identity")
    print("PASS historical baseline superseded; matched LFE lineage remains open")


if __name__ == "__main__":
    main()
