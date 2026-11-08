from __future__ import annotations

import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "DEE-ATMOS-DDP-TIMECODE-FRAME-RATE-BOUNDARY-PARTIAL-V1"
STATUS = "SCHEMA_ENUM_BOUND_HOST_TO_DDP_OWNER_JOIN_UNBOUND_RUNTIME_NOT_RUN"
EXCLUDED = {"MANIFEST.json", "MANIFEST.sha256", "package_seal.json"}
TOKENS = ["not_indicated", "23.976", "24", "25", "29.97", "30", "48", "50", "59.94", "60"]


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
    require(isinstance(files, list) and files, "payload")
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
            require(name == "CHECK.py", "only checker script")
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
    require(seal.get("files") == files, "seal inventory")

    evidence = read_json("EVIDENCE.json")
    require(evidence.get("status") == STATUS, "evidence status")
    require(evidence.get("implementation_ready") == "NO", "evidence readiness")
    require(evidence["ddp_schema"]["enum_tokens"] == TOKENS, "schema tokens")
    require(evidence["ddp_schema"]["default"] == "not_indicated", "schema default")
    require(evidence["host_setter_candidate"]["receiver_to_selected_ddp_output_node"] == "UNBOUND", "host owner")
    require(evidence["ddp_reader_candidate"]["reader_caller_owner"] == "UNBOUND", "reader owner")
    require(evidence["first_unbound_edge"]["status"] == "UNBOUND", "first unbound edge")
    require(evidence["execution"]["dee_started"] is False, "runtime")
    vectors = read_json("INPUT-ONLY-VECTORS.json")
    require(vectors["executed"] is False, "vectors not executed")
    require(vectors["enum_tokens"] == TOKENS, "vector tokens")
    require(vectors["runtime_admission"] == "UNBOUND", "runtime admission")
    parents = read_json("SOURCE-MANIFESTS.json")
    require(len(parents["reused_source_packages"]) == 3, "parent receipt count")
    require(all(item["wash_replayed_this_turn"] is True for item in parents["reused_source_packages"]), "parent replay")
    report = (ROOT / "REPORT.md").read_text(encoding="utf-8")
    require("UNBOUND" in report and "NOT_RUN" in report, "boundaries")
    print(f"PASS package={PACKAGE_ID}")
    print(f"PASS manifest_sha256={digest}")
    print("PASS schema-only tokens and unbound owner transfer")


if __name__ == "__main__":
    main()
