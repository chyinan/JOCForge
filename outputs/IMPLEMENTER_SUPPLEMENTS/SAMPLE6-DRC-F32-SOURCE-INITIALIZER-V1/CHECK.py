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
    init = evidence["source_context"]["constructor_initialization"]
    require(init["word_count"] == 30 and init["typed_interpretation"] == "positive-zero F32", "initializer facts mismatch")
    require(init["row_shape"] == "6 blocks x 5 F32" and init["block_stride_bytes"] == 20, "row layout mismatch")
    require(evidence["source_context"]["later_nonzero_writer"] == "UNBOUND", "writer boundary overclaimed")
    require(evidence["profile_boundary"]["xml_string_to_selector"] == "UNBOUND", "XML selector overclaimed")
    require(evidence["same_static_call_context"]["static_state_base_relation"] == "BOUND", "static state-base join missing")
    require(evidence["same_static_call_context"]["runtime_frame_or_generation"] == "NOT_OBSERVED", "runtime generation overclaimed")
    require(vector["execution"] == "NOT_RUN", "vector execution changed")
    require(vector["later_nonzero_writer"] == "UNBOUND", "vector writer boundary overclaimed")
    require(evidence["execution"]["raw_pcm_included"] is False, "PCM scope changed")
    require(evidence["execution"]["private_key_or_license_material_included"] is False, "privacy scope changed")
    print(f"PASS manifest={manifest_hash}")
    print("PASS 30-word zero initialization, six-by-five F32 rows, and same-state static call edge")
    print("PASS later writer, XML selector, runtime generation, and values remain unresolved")


if __name__ == "__main__":
    main()
