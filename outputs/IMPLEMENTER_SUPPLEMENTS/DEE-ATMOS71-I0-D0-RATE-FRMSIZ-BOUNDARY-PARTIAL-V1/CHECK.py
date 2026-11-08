from __future__ import annotations

import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "DEE-ATMOS71-I0-D0-RATE-FRMSIZ-BOUNDARY-PARTIAL-V1"
STATUS = "TWO_FRAME_RATE_ALLOCATION_STATIC_WRITER_AND_BSI_UNBOUND_RUNTIME_NOT_RUN"
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
    require((ROOT / "MANIFEST.sha256").read_text(encoding="ascii").strip() == f"{digest} MANIFEST.json", "manifest digest")
    seal = read_json("package_seal.json")
    require(seal.get("schema") == "jocforge-implementer-supplement-seal.v1", "seal schema")
    require(seal.get("package_id") == PACKAGE_ID, "seal id")
    require(seal.get("manifest_sha256") == digest, "seal manifest")
    require(seal.get("files") == files, "seal inventory")

    evidence = read_json("EVIDENCE.json")
    require(evidence.get("status") == STATUS, "evidence status")
    require(evidence.get("implementation_ready") == "NO", "evidence readiness")
    rate = evidence["target_rate_arithmetic"]
    require((rate["i0"]["rate_kbps"], rate["d0"]["rate_kbps"], rate["aggregate_au"]["rate_kbps"]) == (640, 1024, 1664), "rate pair")
    require((rate["i0"]["frame_bytes"], rate["d0"]["frame_bytes"], rate["aggregate_au"]["bytes"]) == (2560, 4096, 6656), "frame sizes")
    require(rate["single_1664_frame"]["fits_11_bit_field"] is False, "no single 1664 frame")
    writer = evidence["native_writer_boundary"]
    require(writer["setup_to_writer_owner_join"] == "UNBOUND", "writer join")
    require(writer["bsi_bitrate_field"] == "UNBOUND", "BSI field")
    require(writer["second_word_frmsiz_producer"] == "UNBOUND", "frmsiz writer")
    require(evidence["execution"]["dee_run"] is False, "DEE not run")
    vectors = read_json("INPUT-ONLY-VECTORS.json")
    require(vectors["execution"] == "NOT_RUN", "vectors")
    rows = {item["id"]: item for item in vectors["vectors"]}
    require(rows["single_frame_bound"]["fits_11_bit_frmsiz"] is False, "single-frame field limit")
    require(rows["mode21_rate_allocation_summary_only"]["payload_bytes_included"] is False, "payload exclusion")
    sources = read_json("SOURCE-MANIFESTS.json")
    require(sources["source_package"]["checker_normal"] == "PASS", "source checker")
    require(sources["source_package"]["checker_python_optimized"] == "PASS", "source optimized checker")
    require(sources["reused_rate_pair_package"]["wash_replayed_this_turn"] is True, "parent checker replay")
    require(sources["exclusions"]["mode21_payload_hash_or_bytes"] is False, "failed payload exclusion")
    report = (ROOT / "REPORT.md").read_text(encoding="utf-8")
    require("UNBOUND" in report and "NOT_RUN" in report, "open boundaries")
    print(f"PASS package={PACKAGE_ID}")
    print(f"PASS manifest_sha256={digest}")
    print("PASS 640+1024 two-frame allocation with writer joins open")


if __name__ == "__main__":
    main()
