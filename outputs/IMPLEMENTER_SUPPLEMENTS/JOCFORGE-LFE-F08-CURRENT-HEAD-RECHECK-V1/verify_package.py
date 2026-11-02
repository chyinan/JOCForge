from __future__ import annotations

import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "JOCFORGE-LFE-F08-CURRENT-HEAD-RECHECK-V1"
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
    require(e["current_checkout"]["owner_rs_sha256"] == "46a184023b0b1b33881feb4411f8ed17a0531f52c35658f48664bf46172182ab", "owner source hash")
    require(e["parameter_files"]["lfe_arrays_equal_to_r4_crosswalk_rust_words"] is True, "current parameter arrays")
    require(e["remaining"]["numeric_parity"] == "NOT_RUN", "parity boundary")
    print(f"PASS package={PACKAGE_ID}")
    print(f"PASS manifest_sha256={mhash}")
    print("PASS current HEAD callsite/array identity; numeric parity remains NOT_RUN")


if __name__ == "__main__":
    main()
