from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "SAMPLE6-AU-ALIGNED-JOB-CONTROL-COMPARISON-V2"
DEE_MANIFEST = "9ac30a30cba967a0c4dd264c1ac74eba62c46a84379ac9261fa581fbcfbef668"
RUST_GATE_MANIFEST = "6d5e9a96a90a940bb9d6b7969489b7b71888988d8cc048e62125690f141943a4"
STATUS = "RATE_DRC_DOWNMIX_TRIMS_MATCH_AT_TEMPLATE_LEVEL; OTHER_CONTROLS_PARTIAL"


def require(condition: bool, label: str) -> None:
    if not condition:
        raise SystemExit(f"FAIL {label}")


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def load(name: str) -> dict:
    return json.loads((ROOT / name).read_text(encoding="utf-8"))


def main() -> int:
    manifest = load("MANIFEST.json")
    manifest_hash = sha256(ROOT / "MANIFEST.json")
    require(manifest_hash == (ROOT / "MANIFEST.sha256").read_text(encoding="ascii").split()[0], "manifest hash")
    require(manifest["package_id"] == PACKAGE_ID and manifest["status"] == STATUS, "package/status")
    sources = {row["package_id"]: row for row in manifest["source_packages"]}
    require(sources["native-f06-authorized-adm-job-control-whitelist-addendum"]["manifest_sha256"] == DEE_MANIFEST, "DEE source binding")
    require(sources["SAMPLE6-KEYLESS-DEE-REFERENCE-GATE-V4"]["manifest_sha256"] == RUST_GATE_MANIFEST, "Rust source binding")
    payload = {entry["file"]: entry for entry in manifest["files"]}
    actual = {path.name for path in ROOT.iterdir() if path.is_file() and path.name not in {"MANIFEST.json", "MANIFEST.sha256", "package_seal.json"}}
    require(actual == set(payload), "payload inventory")
    for name, entry in payload.items():
        path = ROOT / name
        require(path.stat().st_size == entry["bytes"], f"size {name}")
        require(sha256(path) == entry["sha256"], f"hash {name}")
    seal = load("package_seal.json")
    sealed = {entry["file"]: entry for entry in seal["files"]}
    require(seal["package_id"] == PACKAGE_ID and seal["file_count"] == len(sealed), "seal identity")
    require(set(sealed) == set(payload) | {"MANIFEST.json", "MANIFEST.sha256"}, "seal inventory")
    for name, entry in sealed.items():
        path = ROOT / name
        require(path.stat().st_size == entry["bytes"], f"sealed size {name}")
        require(sha256(path) == entry["sha256"], f"sealed hash {name}")
    e = load("EVIDENCE.json")
    require(e["dee_job"]["data_rate_kbps"] == e["rust_full_run_source"]["template_audio_controls"]["data_rate_kbps"], "template rate match")
    require(e["dee_job"]["drc"] == e["rust_full_run_source"]["template_audio_controls"]["drc"], "DRC template match")
    require(e["dee_job"]["downmix"]["preferred_downmix_mode"] == e["rust_full_run_source"]["template_audio_controls"]["preferred_downmix_mode"], "downmix mode match")
    require(e["dee_job"]["downmix"]["loro_center_mix_level"] == str(e["rust_full_run_source"]["template_audio_controls"]["mix_levels_db"][0]), "mix levels match")
    require(e["comparison"]["runtime_xml_equivalence"] == "NOT_CAPTURED", "runtime XML boundary")
    require(e["comparison"]["causal_audio_effect_of_remaining_fields"] == "NOT_ESTABLISHED", "causality boundary")
    print(f"PASS package={PACKAGE_ID}")
    print(f"PASS manifest_sha256={manifest_hash}")
    print("PASS explicit rate/DRC/downmix/trims match at source-template level")
    print("PASS runtime XML/acmod/LFE/channel-map equivalence remains open")
    return 0


if __name__ == "__main__":
    main()
