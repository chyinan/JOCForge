from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "FLAT7X-CHANMAP-CANDIDATE-ROW-POINTER-ACCESS-BOUNDARY-V1"
PAYLOAD_NAMES = ("FINDINGS.md", "FIELD-BOUNDARY.json", "SOURCE-RECEIPTS.json", "verify_package.py")


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def read_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def fail(message: str) -> None:
    raise SystemExit(message)


def main() -> None:
    manifest_path = ROOT / "MANIFEST.json"
    manifest_digest = digest(manifest_path)
    declared = (ROOT / "MANIFEST.sha256").read_text(encoding="ascii").strip().split()[0]
    manifest = read_json(manifest_path)
    seal = read_json(ROOT / "PACKAGE-SEAL.json")
    if manifest.get("package_id") != PACKAGE_ID or seal.get("package_id") != PACKAGE_ID:
        fail("package identity mismatch")
    if declared != manifest_digest or seal.get("manifest_sha256") != manifest_digest:
        fail("manifest digest mismatch")

    payloads = {item["path"]: item for item in manifest.get("payloads", [])}
    if set(payloads) != set(PAYLOAD_NAMES):
        fail("manifest payload inventory mismatch")
    for name in PAYLOAD_NAMES:
        path = ROOT / name
        item = payloads[name]
        if not path.is_file() or path.stat().st_size != item.get("size_bytes") or digest(path) != item.get("sha256"):
            fail(f"payload integrity mismatch: {name}")

    boundary = read_json(ROOT / "FIELD-BOUNDARY.json")
    native = boundary.get("native_static_boundary", {})
    policy = boundary.get("implementation_policy", {})
    if native.get("candidate_row_pointer_read_before_mask_setter") is not False:
        fail("candidate pointer access boundary mismatch")
    if native.get("native_per_row_mapping") != "UNBOUND":
        fail("native row mapping boundary was upgraded")
    if policy.get("directspeakers_bed_row_map") != "PROVISIONAL_REPLACEABLE":
        fail("provisional role-map policy mismatch")
    if policy.get("adm_rows_equal_chanmap_locations") is not False:
        fail("dimension-separation invariant mismatch")

    sealed = {item["path"]: item for item in seal.get("files", [])}
    sealed_names = set(PAYLOAD_NAMES) | {"MANIFEST.json", "MANIFEST.sha256"}
    if set(sealed) != sealed_names:
        fail("seal inventory mismatch")
    for name in sealed_names:
        path = ROOT / name
        item = sealed[name]
        if not path.is_file() or path.stat().st_size != item.get("size_bytes") or digest(path) != item.get("sha256"):
            fail(f"sealed file integrity mismatch: {name}")

    print(f"PACKAGE_CHECK=PASS package={PACKAGE_ID} payloads={len(PAYLOAD_NAMES)} status={manifest.get('status')}")


if __name__ == "__main__":
    main()
