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
    require(evidence["plugin_image"]["sha256"] == "3d66bcec36031fd48e6565d15f05fea656642377ca4f8c98cd3ce1cce8b7e95d2", "plugin image mismatch")
    require(evidence["core_image"]["sha256"] == "0cf23520799f52323b0994e5fcc16924da7b027a5b83a0cff3b4f06792c18532", "core image mismatch")
    require(evidence["plugin_output_buffer"]["consumer_receives_plugin_f32_row_pointer"] is False, "plugin output buffer boundary changed")
    require(evidence["core_owner_boundary"]["typed_plugin_to_core_edge"] == "UNBOUND", "host/core edge overclaimed")
    require(evidence["core_owner_boundary"]["global_absence_claim"] is False, "global absence overclaimed")
    require(evidence["execution"]["new_dee_run"] is False, "execution scope changed")
    require(evidence["execution"]["private_key_or_license_material_included"] is False, "privacy scope changed")
    print(f"PASS manifest={manifest_hash}")
    print("PASS plugin callback/output-buffer boundary")
    print("PASS no global host/core absence claim; typed core transfer remains unbound")


if __name__ == "__main__":
    main()
