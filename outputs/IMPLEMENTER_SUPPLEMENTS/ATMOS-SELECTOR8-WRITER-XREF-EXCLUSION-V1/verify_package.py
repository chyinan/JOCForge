from __future__ import annotations

import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "ATMOS-SELECTOR8-WRITER-XREF-EXCLUSION-V1"
PAYLOADS = ["REPORT.md", "EVIDENCE.json", "INPUT-ONLY-VECTOR.json", "SOURCE-MANIFESTS.json", "verify_package.py", "seal_package.py"]


def require(value: bool, message: str) -> None:
    if not value:
        raise SystemExit(f"FAIL: {message}")


def digest(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for block in iter(lambda: f.read(1024 * 1024), b""):
            h.update(block)
    return h.hexdigest()


def main() -> None:
    mp = ROOT / "MANIFEST.json"
    manifest = json.loads(mp.read_text(encoding="utf-8"))
    seal = json.loads((ROOT / "package_seal.json").read_text(encoding="utf-8"))
    mhash = digest(mp)
    require((ROOT / "MANIFEST.sha256").read_text(encoding="ascii").split() == [mhash, "MANIFEST.json"], "manifest checksum")
    require(manifest["package_id"] == seal["package_id"] == PACKAGE_ID, "package id")
    require(manifest["files"] == seal["files"], "seal file list")
    require([x["name"] for x in manifest["files"]] == PAYLOADS, "payload list")
    require(seal["manifest_sha256"] == mhash, "seal manifest hash")
    for item in manifest["files"]:
        p = ROOT / item["name"]
        require(p.stat().st_size == item["bytes"] and digest(p) == item["sha256"], f"payload {item['name']}")
    e = json.loads((ROOT / "EVIDENCE.json").read_text(encoding="utf-8"))
    require(e["writer_xrefs"]["function"] == "FUN_1804d6980", "target writer")
    require(e["bounded_candidate_scan"]["slot_target_is_FUN_1804d6980"] is False, "rejected candidates")
    require(e["remaining_unbound"]["selected_atmos_storage_master_s_selector8_callsite"] == "UNBOUND", "owner boundary")
    require(e["execution_scope"]["production_allowed"] is False, "static-only scope")
    print(f"PASS package={PACKAGE_ID}")
    print(f"PASS manifest_sha256={mhash}")
    print("PASS candidate xrefs excluded; selected writer caller remains UNBOUND")


if __name__ == "__main__":
    main()
