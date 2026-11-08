from __future__ import annotations

import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "DEE-JOC-DOWNMIX-RECONSTRUCTION-PROFILE-BOUNDARY-PARTIAL-V1"
STATUS = "DOWNMIX_CONTROLS_AND_RECONSTRUCTION_PROFILE_DISTINCT_SERIALIZER_JOIN_UNBOUND"
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
    require((ROOT / "MANIFEST.sha256").read_text(encoding="ascii").strip() == f"{digest} MANIFEST.json", "manifest digest")
    seal = read_json("package_seal.json")
    require(seal.get("schema") == "jocforge-implementer-supplement-seal.v1", "seal schema")
    require(seal.get("package_id") == PACKAGE_ID, "seal id")
    require(seal.get("manifest_sha256") == digest, "seal manifest")
    require(seal.get("files") == files, "seal inventory")
    evidence = read_json("EVIDENCE.json")
    require(evidence.get("status") == STATUS, "evidence status")
    require(evidence.get("implementation_ready") == "NO", "evidence readiness")
    five = evidence["five_one_selected_profile4"]
    require((five["config_mode"], five["phase_flag"], five["carrier_count"]) == (0, 1, 5), "legacy controls")
    require(evidence["five_one_full_file_joc_parse"]["profile_index"] == 3, "legacy profile index")
    official = evidence["official_seven_one_structure"]
    require((official["profile_index"], official["profile_name"], official["reconstruction_carrier_count"]) == (1, "flat7x", 7), "official profile")
    require(evidence["legacy_composite_enum"]["index3_component"] == "UNBOUND between downmix_index and reconstruction_profile_index", "composite gap")
    require(evidence["target_7_1"]["mode_control"] == "UNBOUND", "target mode")
    require(evidence["target_7_1"]["algorithm_phase_flag"] == "UNBOUND", "target phase")
    require(evidence["target_7_1"]["serializer_join"] == "UNBOUND", "serializer")
    require(evidence["execution"]["dee_run"] is False, "runtime")
    vectors = read_json("INPUT-ONLY-VECTORS.json")
    require(vectors["execution"] == "NOT_RUN", "vectors")
    parents = read_json("SOURCE-MANIFESTS.json")
    require(parents["source_package"]["checker_normal"] == "PASS", "source checker")
    require(parents["source_package"]["checker_python_optimized"] == "PASS", "source optimized checker")
    require(parents["wash_review"]["implementation_readiness"] == "NO", "source readiness")
    print(f"PASS package={PACKAGE_ID}")
    print(f"PASS manifest_sha256={digest}")
    print("PASS selector dimension split with serializer and target controls left unbound")


if __name__ == "__main__":
    main()
