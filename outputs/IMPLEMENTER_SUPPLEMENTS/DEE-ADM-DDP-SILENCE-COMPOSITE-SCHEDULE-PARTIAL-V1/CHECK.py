from __future__ import annotations

import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "DEE-ADM-DDP-SILENCE-COMPOSITE-SCHEDULE-PARTIAL-V1"
STATUS = "STATIC_SAMPLE_SCHEDULE_BOUND_PCM_MATERIALIZATION_UNBOUND_RUNTIME_NOT_RUN"
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
    require(all(isinstance(name, str) for name in names), "payload paths")
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
            require(name == "CHECK.py", "only checker script allowed")
        data = (ROOT / name).read_bytes()
        require(len(data) == item.get("bytes"), f"length {name}")
        require(sha256(data) == item.get("sha256"), f"hash {name}")
    require(
        (ROOT / "MANIFEST.sha256").read_text(encoding="ascii").strip()
        == f"{digest} MANIFEST.json",
        "manifest digest file",
    )
    seal = read_json("package_seal.json")
    require(seal.get("schema") == "jocforge-implementer-supplement-seal.v1", "seal schema")
    require(seal.get("package_id") == PACKAGE_ID, "seal package id")
    require(seal.get("manifest_sha256") == digest, "seal digest")
    require(seal.get("files") == files, "seal file list")

    evidence = read_json("EVIDENCE.json")
    require(evidence.get("status") == STATUS, "evidence status")
    require(evidence.get("implementation_ready") == "NO", "evidence readiness")
    require(evidence["xml_fields"]["cross_snapshot_default"]["omitted_element_overlay"] == "UNBOUND", "XSD default boundary")
    schedule = evidence["composite_schedule"]
    require(schedule["source_range_length"] == "endSample - startSample", "source range")
    require(schedule["composite_source_start"] == "prior composite length + prepend samples", "prepend placement")
    require("append" in schedule["next_composite_length"], "append placement")
    frame = evidence["first_frame_boundary"]
    require(frame["concrete_reader_target"] == "UNBOUND", "reader target")
    require(frame["zero_sample_materialization"] == "UNBOUND", "zero fill")
    require(frame["pts_or_encoded_track_length_effect"] == "NOT_OBSERVED", "PTS/output effect")
    require(evidence["execution"]["dee_launched"] is False, "DEE not run")
    minimum = evidence["minimum_experiment"]
    require(minimum["status"] == "PROPOSED_NOT_RUN_WAITING_FOR_WASH_TIMELINE_GO", "experiment status")
    require(minimum["timeline_go"] == "NOT_GRANTED", "experiment GO")
    require(minimum["output_temp_paths"] == "UNASSIGNED", "experiment paths")

    vectors = read_json("INPUT-ONLY-VECTORS.json")
    require(vectors["status"] == "INPUT_ONLY_NOT_RUN", "vector status")
    by_name = {item["name"]: item for item in vectors["vectors"]}
    require(by_name["prepend_half_sample_conversion"]["prepend_samples_static"] == 187, "prepend truncation vector")
    require(by_name["append_half_sample_conversion"]["append_samples_static"] == 187, "append truncation vector")
    sched = by_name["schedule_around_selected_source_span"]
    require(sched["composite_source_start_sample_static"] == 6000, "composite start vector")
    require(sched["next_composite_length_samples_static"] == 12000, "composite length vector")
    report = (ROOT / "REPORT.md").read_text(encoding="utf-8")
    require("UNBOUND" in report and "NOT_RUN" in report, "boundary labels")
    print(f"PASS package={PACKAGE_ID}")
    print(f"PASS manifest_sha256={digest}")
    print("PASS schedule order with PCM materialization and output effects open")


if __name__ == "__main__":
    main()
