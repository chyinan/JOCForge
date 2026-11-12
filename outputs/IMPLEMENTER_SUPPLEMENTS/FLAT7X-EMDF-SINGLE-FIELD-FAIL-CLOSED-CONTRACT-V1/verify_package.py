from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "FLAT7X-EMDF-SINGLE-FIELD-FAIL-CLOSED-CONTRACT-V1"
PAYLOAD_NAMES = ("FINDINGS.md", "FIELD-CONTRACT.json", "SOURCE-RECEIPTS.json", "verify_package.py")


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def read_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def fail(message: str) -> None:
    raise SystemExit(message)


def main() -> None:
    manifest_path = ROOT / "MANIFEST.json"
    manifest_digest = digest(manifest_path)
    declared_digest = (ROOT / "MANIFEST.sha256").read_text(encoding="ascii").strip().split()[0]
    manifest = read_json(manifest_path)
    seal = read_json(ROOT / "PACKAGE-SEAL.json")

    if manifest.get("package_id") != PACKAGE_ID or seal.get("package_id") != PACKAGE_ID:
        fail("package identity mismatch")
    if declared_digest != manifest_digest or seal.get("manifest_sha256") != manifest_digest:
        fail("manifest digest mismatch")

    payload_map = {item["path"]: item for item in manifest.get("payloads", [])}
    if set(payload_map) != set(PAYLOAD_NAMES):
        fail("manifest payload inventory mismatch")
    for name in PAYLOAD_NAMES:
        path = ROOT / name
        item = payload_map[name]
        if not path.is_file() or path.stat().st_size != item.get("size_bytes") or digest(path) != item.get("sha256"):
            fail(f"payload integrity mismatch: {name}")

    contract = read_json(ROOT / "FIELD-CONTRACT.json")
    target = contract.get("target_contract", {})
    native = contract.get("native_candidate_boundary", {})
    if target.get("block_ordinal") != 2 or target.get("field_width_bits") != 9:
        fail("target skip-field mismatch")
    if target.get("maximum_container_length_bytes") != 511 or target.get("overflow_behavior") != "REJECT":
        fail("target capacity policy mismatch")
    if target.get("fragmentation_across_fields_blocks_or_aus") is not False:
        fail("single-field policy mismatch")
    if native.get("native_field_capacity_bytes") != "UNBOUND" or native.get("native_overflow_behavior") != "UNBOUND":
        fail("native boundary was upgraded")

    seal_files = {item["path"]: item for item in seal.get("files", [])}
    sealed_names = set(PAYLOAD_NAMES) | {"MANIFEST.json", "MANIFEST.sha256"}
    if set(seal_files) != sealed_names:
        fail("seal inventory mismatch")
    for name in sealed_names:
        path = ROOT / name
        item = seal_files[name]
        if not path.is_file() or path.stat().st_size != item.get("size_bytes") or digest(path) != item.get("sha256"):
            fail(f"sealed file integrity mismatch: {name}")

    print(f"PACKAGE_CHECK=PASS package={PACKAGE_ID} payloads={len(PAYLOAD_NAMES)} status={manifest.get('status')}")


if __name__ == "__main__":
    main()
