from __future__ import annotations

import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "DEE-ATMOS71-I0-D0-AU-TRANSACTION-BOUNDARY-PARTIAL-V1"
STATUS = "PER_BLOCK_JOC_PATH_BOUND_STREAM_ROLE_AU_ORDER_AND_COMMIT_UNBOUND_RUNTIME_NOT_RUN"
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
    require(isinstance(files, list) and files, "files")
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
    require((ROOT / "MANIFEST.sha256").read_text(encoding="ascii").strip() == f"{digest} MANIFEST.json", "manifest digest")
    seal = read_json("package_seal.json")
    require(seal.get("schema") == "jocforge-implementer-supplement-seal.v1", "seal schema")
    require(seal.get("package_id") == PACKAGE_ID, "seal id")
    require(seal.get("manifest_sha256") == digest, "seal digest")
    require(seal.get("files") == files, "seal file list")

    evidence = read_json("EVIDENCE.json")
    require(evidence.get("status") == STATUS, "evidence status")
    require(evidence.get("implementation_ready") == "NO", "evidence readiness")
    native = evidence["native_joc_block_path"]
    require(native["first_two_share_block_ordinal"] is True, "within-block call order")
    require(native["producer_packer_pointer_table_alias"] == "UNBOUND", "pointer alias boundary")
    au = evidence["owner_generation_matrix"]
    require(au["i0_builder_writer"] == "NOT_IDENTIFIED_IN_REUSED_RECEIPTS", "I0 builder")
    require(au["d0_builder_writer"] == "UNBOUND", "D0 builder")
    require(au["i0_then_d0_au_order"] == "UNBOUND", "stream role/order")
    publish = evidence["per_unit_and_file_writer"]
    require(publish["callback_buffer_to_fwrite_buffer_and_length"] == "UNBOUND", "writer join")
    require(publish["durable_au_commit"] == "UNBOUND", "AU commit")
    require(evidence["owner_generation_matrix"]["partial_i0_commit_on_d0_failure"] == "UNBOUND", "partial commit")
    require(evidence["execution"]["dee_run"] is False, "DEE not run")
    vectors = read_json("INPUT-ONLY-VECTORS.json")
    require(vectors["vectors"][0]["status"] == "USER_TARGET_REQUIREMENT", "target is not native evidence")
    require(vectors["vectors"][0]["native_builder_binding"] == "UNBOUND", "builder binding")
    require(vectors["vectors"][1]["status"] == "PROPOSED_NOT_RUN", "failure check")
    sources = read_json("SOURCE-MANIFESTS.json")
    require(len(sources["reused_parent_packages"]) == 4, "parent count")
    require(all(item["wash_replayed_this_turn"] is True for item in sources["reused_parent_packages"]), "parent replay")
    require(sources["target_profile_contract"]["production_allowed"] is False, "production gate")
    report = (ROOT / "REPORT.md").read_text(encoding="utf-8")
    require("UNBOUND" in report and "NOT_RUN" in report, "open boundaries")
    print(f"PASS package={PACKAGE_ID}")
    print(f"PASS manifest_sha256={digest}")
    print("PASS per-block order without inferring stream role or AU commit")


if __name__ == "__main__":
    main()
