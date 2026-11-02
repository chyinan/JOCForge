from __future__ import annotations

import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "DEE-LFE-F08-COEFFICIENT-CANDIDATE-V2"
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
    require(e["current_rust_state"]["owner_calls_f08_lfe_transform_block_after_window_fold"] is True, "current selected F08 callsite")
    require(e["current_rust_state"]["parameter_json_sha256"] == "963a176760f7c7d43dba138553717b7d5273a9457a574f658ee73eeb62e7da5c", "current parameter identity")
    require(e["remaining_limits"]["numeric_parity"] == "NOT_RUN", "numeric parity boundary")
    require(e["remaining_limits"]["adm_physical_row3_to_dee_core_lfe_lane"] == "UNBOUND", "input row boundary")
    require(e["pathway_comparison"]["selected_dee_stride_branch"] == "UNBOUND", "stride boundary")
    print(f"PASS package={PACKAGE_ID}")
    print(f"PASS manifest_sha256={mhash}")
    print("PASS current production parameter mismatch candidate; native branch and numeric parity remain open")


if __name__ == "__main__":
    main()
