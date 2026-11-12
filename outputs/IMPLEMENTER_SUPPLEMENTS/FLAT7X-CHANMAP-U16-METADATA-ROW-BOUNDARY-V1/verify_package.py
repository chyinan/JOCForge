from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "FLAT7X-CHANMAP-U16-METADATA-ROW-BOUNDARY-V1"
PAYLOAD_NAMES = ("FINDINGS.md", "FIELD-BOUNDARY.json", "SOURCE-RECEIPTS.json", "verify_package.py")


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def load(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def fail(message: str) -> None:
    raise SystemExit(message)


def main() -> None:
    manifest_path = ROOT / "MANIFEST.json"
    md = digest(manifest_path)
    declared = (ROOT / "MANIFEST.sha256").read_text(encoding="ascii").strip().split()[0]
    manifest = load(manifest_path)
    seal = load(ROOT / "PACKAGE-SEAL.json")
    if manifest.get("package_id") != PACKAGE_ID or seal.get("package_id") != PACKAGE_ID:
        fail("package identity mismatch")
    if declared != md or seal.get("manifest_sha256") != md:
        fail("manifest digest mismatch")

    payloads = {item["path"]: item for item in manifest.get("payloads", [])}
    if set(payloads) != set(PAYLOAD_NAMES):
        fail("manifest inventory mismatch")
    for name in PAYLOAD_NAMES:
        path = ROOT / name
        entry = payloads[name]
        if not path.is_file() or path.stat().st_size != entry.get("size_bytes") or digest(path) != entry.get("sha256"):
            fail(f"payload integrity mismatch: {name}")

    boundary = load(ROOT / "FIELD-BOUNDARY.json")
    native = boundary.get("native_static_observation", {})
    if native.get("candidate_row_pointer_destination_access") != "U16 write":
        fail("metadata-row field width mismatch")
    if native.get("row_semantic_role") != "UNBOUND" or native.get("join_to_d0_chanmap_0x1a00") != "UNBOUND":
        fail("native row-role join was upgraded")
    if boundary.get("implementation_policy", {}).get("directspeakers_pcm_role_map") != "PROVISIONAL_REPLACEABLE":
        fail("provisional role-map policy mismatch")

    sealed = {item["path"]: item for item in seal.get("files", [])}
    sealed_names = set(PAYLOAD_NAMES) | {"MANIFEST.json", "MANIFEST.sha256"}
    if set(sealed) != sealed_names:
        fail("seal inventory mismatch")
    for name in sealed_names:
        path = ROOT / name
        entry = sealed[name]
        if not path.is_file() or path.stat().st_size != entry.get("size_bytes") or digest(path) != entry.get("sha256"):
            fail(f"sealed file integrity mismatch: {name}")

    print(f"PACKAGE_CHECK=PASS package={PACKAGE_ID} payloads={len(PAYLOAD_NAMES)} status={manifest.get('status')}")


if __name__ == "__main__":
    main()
