from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "FLAT7X-ID14-OBJECT-COUNT-DOWNMIX-AND-COMPLEXITY-FIELDS-V1"
PAYLOAD_NAMES = ("FINDINGS.md", "FIELD-MODEL.json", "SOURCE-RECEIPTS.json", "verify_package.py")


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def load_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def fail(message: str) -> None:
    raise SystemExit(message)


def main() -> None:
    manifest_path = ROOT / "MANIFEST.json"
    manifest_sha_path = ROOT / "MANIFEST.sha256"
    seal_path = ROOT / "PACKAGE-SEAL.json"
    manifest = load_json(manifest_path)
    seal = load_json(seal_path)
    manifest_digest = sha256(manifest_path)
    declared_manifest_digest = manifest_sha_path.read_text(encoding="ascii").strip().split()[0]

    if manifest.get("package_id") != PACKAGE_ID or seal.get("package_id") != PACKAGE_ID:
        fail("package identity mismatch")
    if declared_manifest_digest != manifest_digest:
        fail("manifest digest mismatch")
    if seal.get("manifest_sha256") != manifest_digest:
        fail("seal manifest digest mismatch")

    expected = {item["path"]: item for item in manifest.get("payloads", [])}
    if set(expected) != set(PAYLOAD_NAMES):
        fail("manifest payload inventory mismatch")
    for name in PAYLOAD_NAMES:
        path = ROOT / name
        item = expected[name]
        if not path.is_file() or path.stat().st_size != item.get("size_bytes"):
            fail(f"payload size or presence mismatch: {name}")
        if sha256(path) != item.get("sha256"):
            fail(f"payload digest mismatch: {name}")

    model = load_json(ROOT / "FIELD-MODEL.json")
    target = model.get("target", {})
    native = model.get("native_static_rules", {})
    if target.get("joc_object_count") != 15 or target.get("id14_object_count_bits") != 14:
        fail("target object-count vector mismatch")
    if native.get("id14_object_count", {}).get("encoding_rule") != "count_minus_one":
        fail("ID14 count encoding rule mismatch")
    if target.get("downmix_index") != 1 or target.get("downmix_coefficient_width") != 7:
        fail("downmix vector mismatch")
    if target.get("joc_spatial_index") != 1 or target.get("complexity_index_type_a") != 16:
        fail("Flat7x target profile mismatch")
    if native.get("f12_complexity_candidate", {}).get("target_value_join") != "UNBOUND":
        fail("complexity boundary was upgraded")

    sealed = {item["path"]: item for item in seal.get("files", [])}
    seal_paths = set(PAYLOAD_NAMES) | {"MANIFEST.json", "MANIFEST.sha256"}
    if set(sealed) != seal_paths:
        fail("seal inventory mismatch")
    for name in sorted(seal_paths):
        path = ROOT / name
        item = sealed[name]
        if not path.is_file() or path.stat().st_size != item.get("size_bytes"):
            fail(f"sealed file size or presence mismatch: {name}")
        if sha256(path) != item.get("sha256"):
            fail(f"sealed file digest mismatch: {name}")

    print(f"PACKAGE_CHECK=PASS package={PACKAGE_ID} payloads={len(PAYLOAD_NAMES)} status={manifest.get('status')}")


if __name__ == "__main__":
    main()
