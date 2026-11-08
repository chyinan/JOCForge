from __future__ import annotations

import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "DEE-RF-DRC-PROFILE-CONSUMER-BOUNDARY-PARTIAL-V1"
STATUS = "PARTIAL_SIX_TOKEN_CONDITIONAL_MAP_DYNRNG_JOIN_UNBOUND_RUNTIME_NOT_RUN"
EXCLUDED = {"MANIFEST.json", "MANIFEST.sha256", "package_seal.json"}
EXPECTED = {
    "film_standard": (1, 1),
    "film_light": (0, 2),
    "music_standard": (3, 3),
    "music_light": (2, 4),
    "speech": (4, 5),
    "none": (5, 0),
}


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
    manifest_hash = sha256(manifest_bytes)
    manifest = json.loads(manifest_bytes)
    require(manifest.get("schema") == "jocforge-implementer-supplement.v1", "manifest schema")
    require(manifest.get("package_id") == PACKAGE_ID, "package id")
    require(manifest.get("status") == STATUS, "status")
    require(manifest.get("implementation_ready") == "NO", "readiness")
    files = manifest.get("files")
    require(isinstance(files, list) and files, "payload inventory")
    names = [item.get("path") for item in files]
    require(all(isinstance(name, str) for name in names), "payload names")
    require(len(names) == len(set(names)), "unique payload names")
    actual = sorted(
        item.relative_to(ROOT).as_posix()
        for item in ROOT.rglob("*")
        if item.is_file() and item.name not in EXCLUDED
    )
    require(actual == sorted(names), "exact payload inventory")
    for item in files:
        name = item["path"]
        require(Path(name).suffix.lower() in {".json", ".md", ".py"}, f"artifact type {name}")
        if name.endswith(".py"):
            require(name == "CHECK.py", "only checker source allowed")
        data = (ROOT / name).read_bytes()
        require(len(data) == item.get("bytes"), f"length {name}")
        require(sha256(data) == item.get("sha256"), f"hash {name}")
    require(
        (ROOT / "MANIFEST.sha256").read_text(encoding="ascii").strip()
        == f"{manifest_hash} MANIFEST.json",
        "manifest digest file",
    )
    seal = read_json("package_seal.json")
    require(seal.get("schema") == "jocforge-implementer-supplement-seal.v1", "seal schema")
    require(seal.get("package_id") == PACKAGE_ID, "seal package id")
    require(seal.get("manifest_sha256") == manifest_hash, "seal manifest digest")
    require(seal.get("files") == files, "seal file list")

    evidence = read_json("EVIDENCE.json")
    require(evidence.get("status") == STATUS, "evidence status")
    require(evidence.get("implementation_ready") == "NO", "evidence readiness")
    require(evidence["rf_profile_property"]["runtime_value_writer"] == "UNBOUND", "StringType writer boundary")
    require(evidence["conditional_ddpi_mapping"]["runtime_xml_binding"] == "UNBOUND", "runtime XML boundary")
    require(
        evidence["separate_core_dynrng_path"]["rf_parameter_to_selector_table_owner_mode_generation"] == "UNBOUND",
        "RF/core DRC join",
    )
    require(evidence["compre_boundary"]["native_writer"] == "UNBOUND_IN_SCOPED_TRACE", "compre writer")
    require(evidence["execution"]["runtime"] == "NOT_RUN", "runtime status")

    vectors = read_json("INPUT-ONLY-VECTORS.json")
    require(vectors.get("execution") == "NOT_RUN", "vectors execution")
    rows = vectors.get("vectors", [])
    require(len(rows) == len(EXPECTED), "six token vectors")
    for row in rows:
        token = row.get("token")
        require(token in EXPECTED, f"token {token}")
        require(
            (row.get("static_workflow_u32"), row.get("conditional_ddpi_enc_drcprof_u32")) == EXPECTED[token],
            f"mapping {token}",
        )
        require(row.get("runtime_xml_binding") == "UNBOUND", f"runtime binding {token}")
        require(row.get("observed_output") == "NOT_APPLICABLE", f"output observation {token}")
    require({row["token"] for row in rows} == set(EXPECTED), "token set")
    sources = read_json("SOURCE-MANIFESTS.json")
    require(sources["source_package"]["checker_normal"] == "PASS", "source checker normal")
    require(sources["source_package"]["checker_python_optimized"] == "PASS", "source checker optimized")
    require(
        all(item["payload_hashes_recomputed"] is True for item in sources["verified_parent_packages"]),
        "parent payload hash recomputation",
    )
    report = (ROOT / "REPORT.md").read_text(encoding="utf-8")
    require("UNBOUND" in report and "NOT_RUN" in report, "scope boundary labels")
    print(f"PASS package={PACKAGE_ID}")
    print(f"PASS manifest_sha256={manifest_hash}")
    print("PASS six conditional tokens and open runtime/consumer boundaries")


if __name__ == "__main__":
    main()
