from __future__ import annotations

import hashlib
import json
import struct
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "SAMPLE6-RUST-WINDOW-PROVIDER-SUPERSESSION-V1"
PAYLOADS = {
    "REPORT.md",
    "EVIDENCE.json",
    "INPUT-ONLY-VECTOR.json",
    "SOURCE-MANIFESTS.json",
    "CHECK.py",
}


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
    manifest_hash = digest(manifest_path)
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    require(manifest.get("package_id") == PACKAGE_ID, "package id mismatch")
    require(
        (ROOT / "MANIFEST.sha256").read_text(encoding="utf-8").split()
        == [manifest_hash, "MANIFEST.json"],
        "manifest sidecar mismatch",
    )
    entries = manifest.get("files")
    require(isinstance(entries, list), "manifest files missing")
    require({entry.get("name") for entry in entries} == PAYLOADS, "payload set mismatch")
    for entry in entries:
        path = ROOT / entry["name"]
        require(path.is_file(), f"missing {entry['name']}")
        require(path.stat().st_size == entry.get("bytes"), f"size mismatch: {entry['name']}")
        require(digest(path) == entry.get("sha256"), f"hash mismatch: {entry['name']}")

    sums: dict[str, str] = {}
    for line in sums_path.read_text(encoding="utf-8").splitlines():
        if line.strip():
            file_hash, name = line.split("  ", 1)
            sums[name] = file_hash
    require(set(sums) == PAYLOADS | {"MANIFEST.json"}, "SHA256SUMS scope mismatch")
    for name, file_hash in sums.items():
        require(digest(ROOT / name) == file_hash, f"SHA256SUMS mismatch: {name}")

    seal = json.loads((ROOT / "package_seal.json").read_text(encoding="utf-8"))
    require(seal.get("package_id") == PACKAGE_ID, "seal package mismatch")
    require(seal.get("manifest_sha256") == manifest_hash, "seal manifest mismatch")
    require(seal.get("sha256sums_sha256") == digest(sums_path), "seal sums mismatch")
    require(seal.get("payload_count") == len(PAYLOADS), "seal payload count mismatch")

    evidence = json.loads((ROOT / "EVIDENCE.json").read_text(encoding="utf-8"))
    vector = json.loads((ROOT / "INPUT-ONLY-VECTOR.json").read_text(encoding="utf-8"))
    current = evidence["current_provider"]
    history = evidence["historical_baseline"]
    require(
        evidence["status"].startswith("CURRENT_PROVIDER_MATCHES_NATIVE"),
        "current provider status mismatch",
    )
    require(current["file_sha256"] == "963a176760f7c7d43dba138553717b7d5273a9457a574f658ee73eeb62e7da5c", "provider hash mismatch")
    require(current["window_words_sha256"] == current["native_window_words_sha256"], "current/native table hash mismatch")
    require(current["window_words_sha256"] == "097ea95140024fd143ed18a78299fa72d69bce1c8e0d356aed2efff2a6a099a2", "window hash mismatch")
    repository_root = ROOT.parents[2]
    provider_path = repository_root / current["file"]
    require(provider_path.is_file(), "current Rust provider file missing from repository")
    require(digest(provider_path) == current["file_sha256"], "current Rust provider file hash mismatch")
    provider = json.loads(provider_path.read_text(encoding="utf-8"))
    window_words = provider["fields"]["window"]
    require(len(window_words) == current["window_word_count"], "current Rust window word count mismatch")
    packed_window = b"".join(struct.pack("<I", word) for word in window_words)
    require(hashlib.sha256(packed_window).hexdigest() == current["window_words_sha256"], "current Rust window words hash mismatch")
    require(f"{window_words[21]:08x}" == "3c0ea5aa", "current Rust index-21 word mismatch")
    require(current["index_21"]["native_f32_bits"] == "3c0ea5aa", "native index-21 value mismatch")
    require(current["index_21"]["current_rust_f32_bits"] == "3c0ea5aa", "current Rust index-21 value mismatch")
    require(history["window_words_sha256"] == "45e8cbe4c07bd1b59bc009ed6ac77d7911a3a309b01308131da97e640da95b23", "historical hash mismatch")
    require(history["status"] == "SUPERSEDED_BY_CURRENT_PROVIDER", "historical status mismatch")
    require(vector["provider"]["index_21_f32_bits"] == "3c0ea5aa", "vector provider mismatch")
    require(vector["expected_current_rust_output"]["primary_index_107_f32_bits"] == "3c0ea5aa", "primary vector output mismatch")
    require(vector["expected_current_rust_output"]["companion_index_106_f32_bits"] == "bc0ea5aa", "companion vector output mismatch")
    require(
        evidence["matched_lfe_boundary"]["adm_row_3_to_core_lfe_lane"] == "UNBOUND"
        and evidence["matched_lfe_boundary"]["scratch_history_words_256_to_381_owner"] == "UNBOUND",
        "matched LFE boundary overclaim",
    )
    require(all(value is False for value in evidence["privacy"].values()), "privacy scope mismatch")

    print(f"PASS manifest={manifest_hash}")
    print("PASS current provider hash, window table, and one-hot receipt")
    print("PASS historical baseline superseded; matched LFE lineage remains unbound")


if __name__ == "__main__":
    main()
