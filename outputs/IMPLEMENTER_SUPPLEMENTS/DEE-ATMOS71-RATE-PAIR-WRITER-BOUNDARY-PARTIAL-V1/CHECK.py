from __future__ import annotations

import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "DEE-ATMOS71-RATE-PAIR-WRITER-BOUNDARY-PARTIAL-V1"
STATUS = "TARGET_RATE_PAIR_BOUND_WRITER_AND_D0_AUDIO_UNBOUND_RUNTIME_NOT_RUN"
EXCLUDED = {"MANIFEST.json", "MANIFEST.sha256", "package_seal.json"}


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(f"FAIL {message}")


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def read_json(name: str) -> dict:
    value = json.loads((ROOT / name).read_text(encoding="utf-8"))
    require(isinstance(value, dict), f"JSON object {name}")
    return value


def main() -> None:
    manifest_bytes = (ROOT / "MANIFEST.json").read_bytes()
    digest = sha256(manifest_bytes)
    manifest = json.loads(manifest_bytes)
    require(manifest.get("schema") == "jocforge-implementer-supplement.v1", "manifest schema")
    require(manifest.get("package_id") == PACKAGE_ID, "package id")
    require(manifest.get("status") == STATUS, "status")
    require(manifest.get("implementation_ready") == "NO", "readiness")
    files = manifest.get("files")
    require(isinstance(files, list) and files, "payload files")
    names = [item.get("path") for item in files]
    require(all(isinstance(name, str) for name in names), "paths")
    require(len(names) == len(set(names)), "unique paths")
    actual = sorted(
        item.relative_to(ROOT).as_posix()
        for item in ROOT.rglob("*")
        if item.is_file() and item.name not in EXCLUDED
    )
    require(actual == sorted(names), "exact inventory")
    for item in files:
        name = item["path"]
        require(Path(name).suffix.lower() in {".json", ".md", ".py"}, f"artifact type {name}")
        if name.endswith(".py"):
            require(name == "CHECK.py", "only package checker")
        data = (ROOT / name).read_bytes()
        require(len(data) == item.get("bytes"), f"length {name}")
        require(sha256(data) == item.get("sha256"), f"hash {name}")
    require(
        (ROOT / "MANIFEST.sha256").read_text(encoding="ascii").strip()
        == f"{digest} MANIFEST.json",
        "detached manifest hash",
    )
    seal = read_json("package_seal.json")
    require(seal.get("schema") == "jocforge-implementer-supplement-seal.v1", "seal schema")
    require(seal.get("package_id") == PACKAGE_ID, "seal id")
    require(seal.get("manifest_sha256") == digest, "seal manifest")
    require(seal.get("files") == files, "seal inventory")

    evidence = read_json("EVIDENCE.json")
    require(evidence.get("status") == STATUS, "evidence status")
    require(evidence.get("implementation_ready") == "NO", "evidence readiness")
    require(evidence["existing_native_rate_pair"]["rate_1024"]["condition"] == "only if the lookup is reached", "conditional map")
    writer = evidence["writer_boundary"]
    require(writer["typed_owner_join"] == "UNBOUND", "writer owner")
    require(writer["bsi_bitrate_field"] == "UNBOUND", "BSI field")
    require(writer["frame_size_writer"] == "UNBOUND", "frame-size writer")
    target = evidence["approved_7_1_rate_row"]
    require(target["i0_kbps"] == 640 and target["d0_kbps"] == 1024, "rate allocation")
    require(target["aggregate_access_unit_kbps"] == 1664, "aggregate rate")
    require(target["aggregate_access_unit_bytes"] == 6656, "aggregate size")
    require(target["single_1664_kbps_eac3_frame"] is False, "single-frame rejection")
    require(evidence["production_gates"]["production_allowed"] is False, "production gate")
    require(evidence["execution"]["dee_run"] is False, "runtime")
    sources = read_json("SOURCE-MANIFESTS.json")
    require(sources["source_receipt_package"]["checker_normal"] == "PASS", "source checker")
    require(sources["reused_parent_packages"][0]["wash_replayed_this_turn"] is True, "parent replay")
    require(sources["reused_parent_packages"][1]["checker_replayed_this_turn"] is False, "unavailable checker recorded")
    require(sources["exclusions"]["mode21_ec3_hash_or_bytes"] is False, "mode21 payload exclusion")
    report = (ROOT / "REPORT.md").read_text(encoding="utf-8")
    require("UNBOUND" in report and "NOT_RUN" in report, "boundary labels")
    print(f"PASS package={PACKAGE_ID}")
    print(f"PASS manifest_sha256={digest}")
    print("PASS 640+1024 rate contract with writer/production gates open")


if __name__ == "__main__":
    main()
