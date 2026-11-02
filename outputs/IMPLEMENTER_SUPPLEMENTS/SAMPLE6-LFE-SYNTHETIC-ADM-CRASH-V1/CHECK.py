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
    attempts = evidence["dee_attempts"]
    require(attempts["count"] == 5, "attempt count mismatch")
    require(set(attempts["all_exit_codes"]) == {"0xC0000005"}, "crash codes mismatch")
    require(attempts["all_created_output"] is False, "output boundary changed")
    require(attempts["final_faulting_module"] == "UNOBSERVED", "fault module overclaimed")
    require(evidence["observations"]["au_geometry"] == "NOT_RUN", "AU status changed")
    require(evidence["observations"]["control_vs_tone_output_metrics"] == "NOT_RUN", "metrics status changed")
    require(evidence["source_adm"]["data_bytes_read_or_copied"] == 0, "source data scope changed")
    require(evidence["privacy"]["private_key_or_license_material_included"] is False, "privacy scope changed")
    require(vector["dee_result"] == "0xC0000005; no output", "vector result changed")
    require(vector["audio_comparison"] == "NOT_RUN", "audio comparison overclaimed")
    print(f"PASS manifest={manifest_hash}")
    print("PASS five failed launches, no output, AU/PCM comparisons NOT_RUN")
    print("PASS source data, synthetic media, DBMD payload, and private material excluded")


if __name__ == "__main__":
    main()
