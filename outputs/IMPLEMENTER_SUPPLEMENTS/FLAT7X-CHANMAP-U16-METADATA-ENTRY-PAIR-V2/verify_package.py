from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "FLAT7X-CHANMAP-U16-METADATA-ENTRY-PAIR-V2"
PAYLOAD_NAMES = ("FINDINGS.md", "ENTRY-PAIR.json", "SOURCE-RECEIPTS.json", "verify_package.py")


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def read_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def fail(message: str) -> None:
    raise SystemExit(message)


def main() -> None:
    manifest_path = ROOT / "MANIFEST.json"
    md = digest(manifest_path)
    declared = (ROOT / "MANIFEST.sha256").read_text(encoding="ascii").strip().split()[0]
    manifest = read_json(manifest_path)
    seal = read_json(ROOT / "PACKAGE-SEAL.json")
    if manifest.get("package_id") != PACKAGE_ID or seal.get("package_id") != PACKAGE_ID:
        fail("package identity mismatch")
    if declared != md or seal.get("manifest_sha256") != md:
        fail("manifest digest mismatch")

    payloads = {item["path"]: item for item in manifest.get("payloads", [])}
    if set(payloads) != set(PAYLOAD_NAMES):
        fail("manifest inventory mismatch")
    for name in PAYLOAD_NAMES:
        path = ROOT / name
        item = payloads[name]
        if not path.is_file() or path.stat().st_size != item.get("size_bytes") or digest(path) != item.get("sha256"):
            fail(f"payload integrity mismatch: {name}")

    pair = read_json(ROOT / "ENTRY-PAIR.json").get("observed_candidate_metadata_table", {})
    if pair.get("entry_0", {}).get("write_width_bits") != 16:
        fail("first U16 write width mismatch")
    if pair.get("entry_1", {}).get("offset_bytes") != 2 or pair.get("entry_1", {}).get("write_width_bits") != 16:
        fail("adjacent U16 write mismatch")
    if pair.get("typed_owner") != "UNBOUND" or pair.get("join_to_d0_chanmap_0x1a00") != "UNBOUND":
        fail("semantic join was upgraded")

    sealed = {item["path"]: item for item in seal.get("files", [])}
    seal_names = set(PAYLOAD_NAMES) | {"MANIFEST.json", "MANIFEST.sha256"}
    if set(sealed) != seal_names:
        fail("seal inventory mismatch")
    for name in seal_names:
        path = ROOT / name
        item = sealed[name]
        if not path.is_file() or path.stat().st_size != item.get("size_bytes") or digest(path) != item.get("sha256"):
            fail(f"sealed file integrity mismatch: {name}")

    print(f"PACKAGE_CHECK=PASS package={PACKAGE_ID} payloads={len(PAYLOAD_NAMES)} status={manifest.get('status')}")


if __name__ == "__main__":
    main()
