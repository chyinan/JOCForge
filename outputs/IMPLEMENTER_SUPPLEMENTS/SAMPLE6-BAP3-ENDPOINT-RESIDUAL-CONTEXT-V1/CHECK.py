from __future__ import annotations

import hashlib
import json
import math
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
    sidecar_path = ROOT / "MANIFEST.sha256"
    manifest_hash = digest(manifest_path)
    require(
        sidecar_path.read_text(encoding="utf-8").split() == [manifest_hash, "MANIFEST.json"],
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
    expected_sums = PAYLOADS | {"MANIFEST.json"}
    require(set(sums) == expected_sums, "SHA256SUMS scope mismatch")
    for name, file_hash in sums.items():
        require(digest(ROOT / name) == file_hash, f"SHA256SUMS mismatch: {name}")

    seal = json.loads(seal_path.read_text(encoding="utf-8"))
    require(seal.get("manifest_sha256") == manifest_hash, "seal manifest mismatch")
    require(seal.get("sha256sums_sha256") == digest(sums_path), "seal sums mismatch")
    require(seal.get("payload_count") == len(PAYLOADS), "seal payload count mismatch")

    evidence = json.loads((ROOT / "EVIDENCE.json").read_text(encoding="utf-8"))
    vector = json.loads((ROOT / "INPUT-ONLY-VECTOR.json").read_text(encoding="utf-8"))
    require("UNOBSERVED" in evidence["status"] and "UNBOUND" in evidence["status"], "status overclaim")
    require(evidence["observation_status"]["raw7_occurrence_in_window"] == "UNOBSERVED", "raw-7 status changed")
    require(evidence["observation_status"]["pcm_residual_causality"] == "UNBOUND", "causality status changed")
    require(vector.get("input_only") is True, "vector is not input-only")
    require(vector.get("native_runtime_execution") == "NOT_RUN", "native runtime status changed")
    require(vector.get("rust_runtime_execution") == "NOT_RUN", "Rust runtime status changed")
    require(evidence["privacy"] == {
        "private_key_or_license_material_included": False,
        "raw_pcm_included": False,
        "encoded_payload_included": False,
        "native_code_or_disassembly_included": False,
    }, "privacy scope changed")

    quantizer = vector["quantizer_call_boundary"]
    require(quantizer["bap_row"] == [3] and quantizer["decoded_exponent"] == [0], "vector inputs mismatch")
    require(quantizer["coefficient_row_f32_bits"] == ["0x3f800000"], "coefficient bits mismatch")
    raw_code = math.trunc((1.0 + 1.0) * evidence["native_rule"]["bap3_scale_f32"])
    clamped_code = min(raw_code, evidence["native_rule"]["bap3_level_count"] - 1)
    require(raw_code == 7 and clamped_code == 6, "BAP-3 endpoint arithmetic mismatch")
    require(vector["expected_native"]["packed_code_bits"] == "110", "native packed code mismatch")
    print(f"PASS manifest={manifest_hash}")
    print("PASS payload hashes, sizes, and seal")
    print("PASS native BAP-3 endpoint vector; sample hit and PCM causality remain open")


if __name__ == "__main__":
    main()
