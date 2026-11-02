from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PAYLOADS = ["REPORT.md", "EVIDENCE.json", "SOURCE-MANIFESTS.json", "verify_package.py", "seal_package.py"]
PACKAGE_ID = "F01E-36-ROW-COLD-OWNER-BOUNDARY-V1"


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(f"FAIL: {message}")


def digest(path: Path) -> str:
    value = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            value.update(block)
    return value.hexdigest()


def main() -> None:
    manifest_path = ROOT / "MANIFEST.json"
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    seal = json.loads((ROOT / "package_seal.json").read_text(encoding="utf-8"))
    manifest_hash = digest(manifest_path)
    require((ROOT / "MANIFEST.sha256").read_text(encoding="ascii").split() == [manifest_hash, "MANIFEST.json"], "manifest checksum")
    require(seal.get("manifest_sha256") == manifest_hash, "seal manifest hash")
    require(manifest.get("package_id") == PACKAGE_ID == seal.get("package_id"), "package id")
    require(manifest.get("files") == seal.get("files"), "manifest/seal file list")
    require([item.get("name") for item in manifest["files"]] == PAYLOADS, "payload order")
    for item in manifest["files"]:
        path = ROOT / item["name"]
        require(path.is_file(), f"missing {item['name']}")
        require(path.stat().st_size == item["bytes"], f"size {item['name']}")
        require(digest(path) == item["sha256"], f"hash {item['name']}")
    evidence = json.loads((ROOT / "EVIDENCE.json").read_text(encoding="utf-8"))
    require(evidence["approved_contract"]["cold_object_owners"] == 28, "approved cold owner shape")
    require(evidence["current_wip"]["function"] == "ValidatedF01EColdProvider::for_admitted_adm_input", "WIP projection")
    require(evidence["remaining_gates"]["fresh_zero_callback"] == "requires physical_rows.len() == 38", "fresh-zero gate")
    require(evidence["remaining_gates"]["full_36_row_joc"] == "NOT_RUN", "end-to-end status")
    require(evidence["scope"]["dsp_formula_changed"] is False, "DSP scope")
    print(f"PASS package={PACKAGE_ID}")
    print(f"PASS manifest_sha256={manifest_hash}")
    print("PASS approved cold schema fixed28; dynamic WIP separate; end-to-end NOT_RUN")


if __name__ == "__main__":
    main()
