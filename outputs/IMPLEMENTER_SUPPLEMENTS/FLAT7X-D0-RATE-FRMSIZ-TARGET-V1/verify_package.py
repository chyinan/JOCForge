from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "FLAT7X-D0-RATE-FRMSIZ-TARGET-V1"
PAYLOAD_NAMES = ("FINDINGS.md", "FRAME-SIZE-VECTOR.json", "SOURCE-RECEIPTS.json", "verify_package.py")


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

    payload_map = {item["path"]: item for item in manifest.get("payloads", [])}
    if set(payload_map) != set(PAYLOAD_NAMES):
        fail("manifest payload inventory mismatch")
    for name in PAYLOAD_NAMES:
        path = ROOT / name
        item = payload_map[name]
        if not path.is_file() or path.stat().st_size != item.get("size_bytes") or digest(path) != item.get("sha256"):
            fail(f"payload integrity mismatch: {name}")

    vector = read_json(ROOT / "FRAME-SIZE-VECTOR.json")
    i0 = vector.get("frame_targets", {}).get("i0", {})
    d0 = vector.get("frame_targets", {}).get("d0", {})
    if i0.get("rate_kbps") != 640 or i0.get("frame_bytes") != 2560:
        fail("I0 target vector mismatch")
    if d0.get("rate_kbps") != 1024 or d0.get("frame_bytes") != 4096:
        fail("D0 target vector mismatch")
    if d0.get("frmsiz_field_bits") != 11 or d0.get("frmsiz_value") != 2047:
        fail("D0 frmsiz vector mismatch")
    binding = vector.get("native_binding", {})
    if binding.get("profile_rate_to_typed_d0_slot") != "UNBOUND":
        fail("native profile-rate join was upgraded")

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
