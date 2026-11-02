from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "DEE-CORE-FRAME-PROVIDER-BOUNDARY-V1"
SOURCE_MANIFEST = "51ce0964d527147a2fe52db1cc9b82976a12ef1a8bd4086f84760ccde46506c1"
STATUS = "CORE_BRANCH_PROVIDER_TYPE_BOUND; SELECTED_BACKEND_VALUE_UNBOUND; FORMAT_CONTROLS_UNBOUND; PCM_CAUSALITY_UNBOUND"


def require(condition: bool, label: str) -> None:
    if not condition:
        raise SystemExit(f"FAIL {label}")


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def load(name: str) -> dict:
    return json.loads((ROOT / name).read_text(encoding="utf-8"))


def main() -> None:
    manifest = load("MANIFEST.json")
    manifest_hash = sha256(ROOT / "MANIFEST.json")
    require(manifest_hash == (ROOT / "MANIFEST.sha256").read_text(encoding="ascii").split()[0], "manifest hash")
    require(manifest["package_id"] == PACKAGE_ID and manifest["status"] == STATUS, "manifest id/status")
    require(manifest["source_reverse_manifest_sha256"] == SOURCE_MANIFEST, "source manifest")
    entries = {entry["file"]: entry for entry in manifest["files"]}
    actual = {
        path.name for path in ROOT.iterdir()
        if path.is_file() and path.name not in {"MANIFEST.json", "MANIFEST.sha256", "package_seal.json"}
    }
    require(actual == set(entries), "payload inventory")
    for name, entry in entries.items():
        path = ROOT / name
        require(path.stat().st_size == entry["bytes"], f"size {name}")
        require(sha256(path) == entry["sha256"], f"hash {name}")
    seal = load("package_seal.json")
    sealed = {entry["file"]: entry for entry in seal["files"]}
    require(seal["package_id"] == PACKAGE_ID, "seal id")
    require(set(sealed) == set(entries) | {"MANIFEST.json", "MANIFEST.sha256"}, "seal inventory")
    for name, entry in sealed.items():
        path = ROOT / name
        require(path.stat().st_size == entry["bytes"], f"sealed size {name}")
        require(sha256(path) == entry["sha256"], f"sealed hash {name}")
    evidence = load("EVIDENCE.json")
    require(evidence["status"] == STATUS, "evidence status")
    provider = evidence["core_provider_contract"]
    require(provider["normal_request_samples"] == 1536 and provider["flush_request_samples"] == 0, "request sizes")
    require(provider["encoder_process"] == "ddp_joc_pro_encode_process", "core encoder call")
    require(provider["sample_rate_acmod_lfeon_wire_order_fields"] == "UNBOUND", "format fields")
    require(evidence["provider_selection"]["selected_value_for_authorized_job"] == "UNBOUND", "selected provider value")
    require(evidence["pcm_effect"]["pcm_mismatch_explained"] is False, "PCM attribution")
    vector = load("INPUT-ONLY-VECTOR.json")
    require(vector["input"]["request_sample_count"] == 1536 and vector["execution"] == "NOT_RUN", "vector scope")
    print(f"PASS package={PACKAGE_ID}")
    print(f"PASS manifest_sha256={manifest_hash}")
    print("PASS 1536-sample core provider boundary and explicit branch/format/PCM limits")


if __name__ == "__main__":
    main()
