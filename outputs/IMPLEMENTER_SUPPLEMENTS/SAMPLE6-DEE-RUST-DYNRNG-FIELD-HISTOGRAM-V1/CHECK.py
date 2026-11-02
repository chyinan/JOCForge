from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PAYLOADS = {"REPORT.md", "EVIDENCE.json", "SOURCE-MANIFESTS.json", "CHECK.py"}


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
    names = {item.get("name") for item in files}
    require(names == PAYLOADS, "payload set mismatch")
    for item in files:
        path = ROOT / item["name"]
        require(path.is_file(), f"missing {item['name']}")
        require(path.stat().st_size == item.get("bytes"), f"size mismatch: {item['name']}")
        require(digest(path) == item.get("sha256"), f"hash mismatch: {item['name']}")

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
    full = evidence["full_file"]
    window = evidence["target_window"]
    require(evidence["matched_outputs"]["parsed_fields_per_stream"] == 25482, "full-file field count mismatch")
    require(full["dynrnge_presence_mismatches"] == 0, "full-file presence mismatch")
    require(sum(full["per_block_u8_difference_counts"]) == 10284, "full-file U8 count mismatch")
    require(window["fields_per_stream"] == 750, "window field count mismatch")
    require(window["dynrnge_presence_mismatches"] == 0, "window presence mismatch")
    require(sum(window["per_block_u8_difference_counts"]) == 194, "window U8 count mismatch")
    require(window["all_differences"] == {"dee_u8": 1, "rust_u8": 0}, "window code pair mismatch")
    require(evidence["parser_scope"]["raw_syncframe_persisted"] is False, "syncframe persistence scope changed")
    require(evidence["parser_scope"]["pcm_generated_or_persisted"] is False, "PCM scope changed")
    require(evidence["parser_scope"]["private_key_or_license_accessed"] is False, "privacy scope changed")
    require(evidence["causality"]["effect_when_decoder_drc_enabled"] == "UNMEASURED", "rendered effect overclaimed")
    print(f"PASS manifest={manifest_hash}")
    print("PASS full-file and target-window six-block field counts")
    print("PASS parser, privacy, and causality limits")


if __name__ == "__main__":
    main()
