from __future__ import annotations

import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "DEE-ATMOSMEZZ-EXPLICIT-OFFSET-FFOA-READER-ALIGNMENT-PARTIAL-V1"
STATUS = "PARTIAL_READER_ALIGNMENT_UNBOUND_SOURCE_FIELDS_PARTIAL_EXPERIMENT_NOT_RUN"
EXCLUDED = {"MANIFEST.json", "MANIFEST.sha256", "package_seal.json"}


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(f"FAIL {message}")


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def read_json(name: str) -> dict:
    value = json.loads((ROOT / name).read_text(encoding="utf-8"))
    require(isinstance(value, dict), f"JSON object: {name}")
    return value


def main() -> None:
    manifest_bytes = (ROOT / "MANIFEST.json").read_bytes()
    manifest_hash = sha256(manifest_bytes)
    manifest = json.loads(manifest_bytes)
    require(manifest.get("schema") == "jocforge-implementer-supplement.v1", "manifest schema")
    require(manifest.get("package_id") == PACKAGE_ID, "package id")
    require(manifest.get("audience") == "IMPLEMENTATION_EVIDENCE_ONLY", "audience")
    require(manifest.get("status") == STATUS, "status")
    require(manifest.get("implementation_ready") == "NO", "readiness")
    files = manifest.get("files")
    require(isinstance(files, list) and files, "payload inventory")
    names = [entry.get("path") for entry in files]
    require(all(isinstance(name, str) for name in names), "payload paths")
    require(len(names) == len(set(names)), "unique payload paths")
    actual = sorted(
        path.relative_to(ROOT).as_posix()
        for path in ROOT.rglob("*")
        if path.is_file() and path.name not in EXCLUDED
    )
    require(actual == sorted(names), "manifest covers exact inventory")
    for entry in files:
        name = entry["path"]
        require(Path(name).suffix.lower() in {".json", ".md", ".py"}, f"artifact type {name}")
        if name.endswith(".py"):
            require(name == "CHECK.py", "checker is only Python payload")
        data = (ROOT / name).read_bytes()
        require(len(data) == entry.get("bytes"), f"length {name}")
        require(sha256(data) == entry.get("sha256"), f"hash {name}")
    require(
        (ROOT / "MANIFEST.sha256").read_text(encoding="ascii").strip()
        == f"{manifest_hash} MANIFEST.json",
        "detached manifest digest",
    )
    seal = read_json("package_seal.json")
    require(seal.get("schema") == "jocforge-implementer-supplement-seal.v1", "seal schema")
    require(seal.get("package_id") == PACKAGE_ID, "seal package id")
    require(seal.get("manifest_sha256") == manifest_hash, "seal manifest hash")
    require(seal.get("files") == files, "seal payload inventory")

    evidence = read_json("EVIDENCE.json")
    require(evidence.get("status") == STATUS, "evidence status")
    require(evidence.get("implementation_ready") == "NO", "evidence readiness")
    obs = evidence["observations"]
    converter = obs["explicit_value_converter"]
    require(converter["converter"] == "FUN_1401359e0", "converter identity")
    require(converter["return_unit"] == "seconds", "converter unit")
    require(converter["dotted_form"]["fraction_rule"].startswith("prefix 0."), "fraction rule")
    norm = obs["normalization"]
    require(norm["explicit_offset"].startswith("replaces selected"), "offset rule")
    require(norm["final_ffoa_rule"] == "max(selected_offset, converted_explicit_ffoa)", "FFOA clamp")
    reader = obs["ddp_setup_and_reader"]
    require(reader["explicit_option_to_cursor_initial_value_or_seek"] == "UNBOUND", "cursor boundary")
    require(reader["explicit_option_to_duration_clipping"] == "UNBOUND", "duration boundary")
    src = obs["source_metadata"]
    require(src["adm_start_time"]["exact_xml_element_or_attribute"] == "UNBOUND", "start field")
    require(src["ffoa_source"]["typed_adm_bwf_writer"] == "UNBOUND", "FFOA producer")
    require(evidence["minimum_experiment"]["status"] == "PROPOSED_NOT_RUN", "experiment status")
    require(evidence["execution"]["dee_executed"] is False, "DEE not executed")
    require(evidence["execution"]["experiment"] == "NOT_RUN", "execution status")

    vectors = read_json("INPUT-ONLY-VECTORS.json")
    require(len(vectors.get("vectors", [])) == 5, "input-only vector count")
    require(
        all(item.get("status") in {"MODEL_ARITHMETIC_ONLY", "REUSED_NOT_RESCANNED", "PROPOSED_NOT_RUN"}
            for item in vectors["vectors"]),
        "vectors remain non-runtime",
    )
    sources = read_json("SOURCE-MANIFESTS.json")
    require(sources["source_package"]["checker_normal"] == "PASS", "source checker normal")
    require(sources["source_package"]["checker_python_optimized"] == "PASS", "source checker optimized")
    require(
        all(item["manifest_sha256_recomputed"] is True for item in sources["reused_source_packages"]),
        "parent manifest hashes recomputed",
    )
    require(
        all(item["checker_normal"] == "PASS" and item["checker_python_optimized"] == "PASS"
            for item in sources["reused_source_packages"]),
        "parent checkers",
    )
    report = (ROOT / "REPORT.md").read_text(encoding="utf-8")
    require("UNBOUND" in report and "PROPOSED_NOT_RUN" in report, "open boundaries recorded")
    print(f"PASS package={PACKAGE_ID}")
    print(f"PASS manifest_sha256={manifest_hash}")
    print("PASS scope, source receipts, and non-runtime boundaries")


if __name__ == "__main__":
    main()
