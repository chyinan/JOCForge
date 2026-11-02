from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
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
    seal_path = ROOT / "package_seal.json"
    manifest_hash = digest(manifest_path)
    require(
        (ROOT / "MANIFEST.sha256").read_text(encoding="utf-8").split()
        == [manifest_hash, "MANIFEST.json"],
        "manifest sidecar mismatch",
    )
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    entries = manifest.get("files")
    require(isinstance(entries, list), "manifest files missing")
    names = {entry.get("name") for entry in entries}
    require(names == PAYLOADS, "payload set mismatch")
    for entry in entries:
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
    require("DECODED_CORE_LFE_GAIN_UNBOUND" in evidence["status"], "core PCM boundary missing")
    require(evidence["capture_b"]["decoded_eac3_lfe_amplitude_gain"] == "NOT_ESTABLISHED", "gain overclaim")
    require(evidence["capture_b"]["effect_on_matched_pcm_residual"] == "UNBOUND", "causality overclaim")
    require(evidence["cross_capture"]["capture_a_b_same_process"] is False, "cross-capture boundary changed")
    require(evidence["symbolic_vector"]["sample_values_materialized"] is False, "vector materialized")
    require(evidence["symbolic_vector"]["expected_output_present"] is False, "expected output overclaim")
    incident = evidence["scope_incident"]
    require(incident["accidental_sample_buffer_read_request"] is True, "incident omitted")
    require(incident["bytes_requested"] == 8, "incident size mismatch")
    require(incident["sample_values_retained_or_used"] is False, "sample values retained or used")
    require(incident["sample_values_in_this_package"] is False, "sample values included")
    require(evidence["privacy"]["pcm_included"] is False, "PCM scope changed")
    require(evidence["privacy"]["private_key_or_license_material_included"] is False, "privacy scope changed")
    require(vector["execution_status"] == "NOT_RUN", "vector execution status changed")
    print(f"PASS manifest={manifest_hash}")
    print("PASS spatial-analysis weight and non-core-gain boundary")
    print("PASS symbolic-only vector, cross-capture limits, and disclosed 8-byte incident")


if __name__ == "__main__":
    main()
