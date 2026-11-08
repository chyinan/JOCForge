from __future__ import annotations

import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "DEE-CUSTOM-TRIM-5-1-FIRST-CONSUMER-BOUNDARY-PARTIAL-V1"
STATUS = "STATIC_ENUM_AND_METADATA_BOUND_FIRST_AUDIO_GAIN_CONSUMER_UNBOUND_RUNTIME_NOT_RUN"
EXCLUDED = {"MANIFEST.json", "MANIFEST.sha256", "package_seal.json"}
CHOICES = {
    "surround_trim_5_1": {"auto", "0", "-3", "-6", "-9"},
    "height_trim_5_1": {"auto", "-3", "-6", "-9", "-12"},
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
    require(len(names) == len(set(names)), "unique files")
    actual = sorted(
        item.relative_to(ROOT).as_posix()
        for item in ROOT.rglob("*")
        if item.is_file() and item.name not in EXCLUDED
    )
    require(actual == sorted(names), "exact package inventory")
    for item in files:
        name = item["path"]
        require(Path(name).suffix.lower() in {".json", ".md", ".py"}, f"artifact type {name}")
        if name.endswith(".py"):
            require(name == "CHECK.py", "only checker is source-like payload")
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
    require(seal.get("package_id") == PACKAGE_ID, "seal id")
    require(seal.get("manifest_sha256") == digest, "seal digest")
    require(seal.get("files") == files, "seal file list")

    evidence = read_json("EVIDENCE.json")
    require(evidence.get("status") == STATUS, "evidence status")
    require(evidence.get("implementation_ready") == "NO", "evidence readiness")
    fields = evidence.get("fields", [])
    require({item["xml_key"] for item in fields} == set(CHOICES), "trim field set")
    for item in fields:
        require(set(item["declared_choices"]) == CHOICES[item["xml_key"]], "declared choices")
        require(item["runtime_token_assignment"] == "UNBOUND", "runtime assignment")
        require(item["numeric_conversion_unit_precision"] == "UNBOUND", "numeric semantics")
        require(item["first_effective_audio_or_bsi_consumer"] == "UNBOUND", "first consumer")
        require(item["effect"] == "NOT_RUN", "effect status")
    require(evidence["dynamic_plan"]["status"] == "PROPOSED_NOT_RUN", "dynamic plan")
    require(evidence["execution"]["dee_or_jocforge"] == "NOT_RUN", "execution")

    vectors = read_json("INPUT-ONLY-VECTORS.json")
    rows = vectors.get("vectors", [])
    require(len(rows) == sum(len(value) for value in CHOICES.values()), "vector count")
    require(
        {(row["field"], row["token"]) for row in rows}
        == {(field, token) for field, tokens in CHOICES.items() for token in tokens},
        "input vector set",
    )
    require(all(row["runtime_assignment"] == "UNBOUND" for row in rows), "runtime vectors")
    report = (ROOT / "REPORT.md").read_text(encoding="utf-8")
    require("UNBOUND" in report and "NOT_RUN" in report, "scope labels")
    print(f"PASS package={PACKAGE_ID}")
    print(f"PASS manifest_sha256={digest}")
    print("PASS trim choice vectors and unresolved audio-consumer boundary")


if __name__ == "__main__":
    main()
