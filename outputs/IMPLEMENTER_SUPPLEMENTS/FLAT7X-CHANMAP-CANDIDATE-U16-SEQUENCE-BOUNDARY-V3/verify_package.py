from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "FLAT7X-CHANMAP-CANDIDATE-U16-SEQUENCE-BOUNDARY-V3"
PAYLOAD_NAMES = ("FINDINGS.md", "SEQUENCE-BOUNDARY.json", "SOURCE-RECEIPTS.json", "verify_package.py")


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

    payload_map = {item["path"]: item for item in manifest.get("payloads", [])}
    if set(payload_map) != set(PAYLOAD_NAMES):
        fail("manifest inventory mismatch")
    for name in PAYLOAD_NAMES:
        path = ROOT / name
        entry = payload_map[name]
        if not path.is_file() or path.stat().st_size != entry.get("size_bytes") or digest(path) != entry.get("sha256"):
            fail(f"payload integrity mismatch: {name}")

    sequence = load(ROOT / "SEQUENCE-BOUNDARY.json").get("observed_sequence", {})
    if sequence.get("store_width_bits") != 16 or sequence.get("destination_stride_bytes") != 2:
        fail("U16 stride boundary mismatch")
    if sequence.get("store_count_in_selected_loop") != 6 or sequence.get("base_register_owner") != "UNBOUND":
        fail("sequence owner/count boundary mismatch")
    joins = load(ROOT / "SEQUENCE-BOUNDARY.json").get("semantic_joins", {})
    if joins.get("d0_chanmap_0x1a00") != "UNBOUND" or joins.get("joc_carriers") != "UNBOUND":
        fail("semantic join was upgraded")

    sealed = {item["path"]: item for item in seal.get("files", [])}
    seal_names = set(PAYLOAD_NAMES) | {"MANIFEST.json", "MANIFEST.sha256"}
    if set(sealed) != seal_names:
        fail("seal inventory mismatch")
    for name in seal_names:
        path = ROOT / name
        entry = sealed[name]
        if not path.is_file() or path.stat().st_size != entry.get("size_bytes") or digest(path) != entry.get("sha256"):
            fail(f"sealed file integrity mismatch: {name}")

    print(f"PACKAGE_CHECK=PASS package={PACKAGE_ID} payloads={len(PAYLOAD_NAMES)} status={manifest.get('status')}")


if __name__ == "__main__":
    main()
