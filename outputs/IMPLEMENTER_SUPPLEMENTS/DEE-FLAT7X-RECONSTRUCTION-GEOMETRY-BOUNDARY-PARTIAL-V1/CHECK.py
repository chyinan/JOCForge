from __future__ import annotations

import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "DEE-FLAT7X-RECONSTRUCTION-GEOMETRY-BOUNDARY-PARTIAL-V1"
STATUS = "PROFILE_SHAPE_BOUND_NATIVE_SELECTOR_AND_GEOMETRY_CONSUMER_UNBOUND"
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
    require(seal.get("manifest_sha256") == digest, "seal manifest hash")
    require(seal.get("files") == files, "seal inventory")

    evidence = read_json("EVIDENCE.json")
    require(evidence.get("status") == STATUS, "evidence status")
    require(evidence.get("implementation_ready") == "NO", "readiness")
    official = evidence["target_profile"]
    require((official["profile"], official["profile_index"], official["reconstruction_carriers"]) == ("flat7x", 1, 7), "flat7x shape")
    require(official["carrier_to_row_or_object_mapping"] == "UNBOUND", "carrier mapping")
    legacy = evidence["legacy_profile"]
    require((legacy["profile_index"], legacy["reconstruction_carriers"]) == (3, 5), "legacy shape")
    require(evidence["bounded_ghidra_queries"]["global_absence_claim"] is False, "bounded search")
    require(evidence["unbound_native_edges"]["native_matrix_consumer"] == "UNBOUND_NOT_TRACED", "matrix edge")
    require(evidence["execution"]["dee_run"] is False, "DEE status")
    vectors = read_json("INPUT-ONLY-VECTORS.json")
    require(vectors["execution"] == "NOT_RUN", "vector execution")
    parents = read_json("SOURCE-MANIFESTS.json")
    require(parents["source_package"]["checker_normal"] == "PASS", "source checker")
    require(parents["source_package"]["checker_python_optimized"] == "PASS", "source optimized checker")
    report = (ROOT / "REPORT.md").read_text(encoding="utf-8")
    require("UNBOUND" in report and "NOT_RUN" in report, "boundary labels")
    print(f"PASS package={PACKAGE_ID}")
    print(f"PASS manifest_sha256={digest}")
    print("PASS flat7x profile shape with native geometry edge open")


if __name__ == "__main__":
    main()
