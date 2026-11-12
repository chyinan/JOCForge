from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "FLAT7X-PER-SPAN-FILEWRITER-SINK-AND-STAGING-CONTRACT-V1"
PAYLOAD_NAMES = ("FINDINGS.md", "WRITER-BOUNDARY.json", "SOURCE-RECEIPTS.json", "verify_package.py")


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
        fail("manifest payload inventory mismatch")
    for name in PAYLOAD_NAMES:
        path = ROOT / name
        entry = payloads[name]
        if not path.is_file() or path.stat().st_size != entry.get("size_bytes") or digest(path) != entry.get("sha256"):
            fail(f"payload integrity mismatch: {name}")

    data = load(ROOT / "WRITER-BOUNDARY.json")
    native = data.get("native_selected_path", {})
    policy = data.get("jocforge_staging_policy", {})
    if native.get("nonzero_span_sink") != "fwrite(buffer, 1, length, FILE*)":
        fail("native per-span sink mismatch")
    if native.get("native_pair_atomicity") != "UNBOUND":
        fail("native atomicity boundary was upgraded")
    if policy.get("append_order") != ["I0", "D0"] or policy.get("publish_only_complete_pair") is not True:
        fail("target pair staging policy mismatch")
    if policy.get("existing_five_one_route_unchanged") is not True:
        fail("5.1 preservation requirement missing")

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
