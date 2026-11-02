from __future__ import annotations

import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "ADM-AUTO-TIME-HOST-TO-CONVERTER-MAP-JOIN-V1"
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
    require(e["host_edge"]["host_map"] == "dee::input::AtmosMezz+0x130", "host map")
    require(e["converter_edge"]["map"] == "PrivContext+0xa8", "converter map")
    require(e["join"]["typed_transfer_found"] is False and e["join"]["same_owner_or_generation"] == "UNBOUND", "cross-module join")
    require(e["mvp_boundary"]["default_file_position_mvp_blocked"] is False, "MVP status")
    require(e["execution_scope"]["production_allowed"] is False, "static-only scope")
    print(f"PASS package={PACKAGE_ID}")
    print(f"PASS manifest_sha256={mhash}")
    print("PASS host writer bound; converter context transfer remains UNBOUND")


if __name__ == "__main__":
    main()
