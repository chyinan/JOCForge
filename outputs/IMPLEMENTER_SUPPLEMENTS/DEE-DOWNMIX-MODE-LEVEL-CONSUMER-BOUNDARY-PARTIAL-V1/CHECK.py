from __future__ import annotations

import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "DEE-DOWNMIX-MODE-LEVEL-CONSUMER-BOUNDARY-PARTIAL-V1"
STATUS = "STATIC_TYPED_CONFIG_BOUND_HOST_WRITER_AND_AUDIO_CONSUMERS_UNBOUND_RUNTIME_NOT_RUN"
EXCLUDED = {"MANIFEST.json", "MANIFEST.sha256", "package_seal.json"}
MODE_CODES = {"loro": 0, "ltrt": 1, "ltrt-pl2": 2, "not_indicated": 3}
LEVEL_CODES = {"+3": 0, "+1.5": 1, "0": 2, "-1.5": 3, "-3": 4, "-4.5": 5, "-6": 6, "-inf": 7}


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
            require(name == "CHECK.py", "only package checker source")
        data = (ROOT / name).read_bytes()
        require(len(data) == item.get("bytes"), f"length {name}")
        require(sha256(data) == item.get("sha256"), f"hash {name}")
    require(
        (ROOT / "MANIFEST.sha256").read_text(encoding="ascii").strip()
        == f"{digest} MANIFEST.json",
        "manifest digest",
    )
    seal = read_json("package_seal.json")
    require(seal.get("schema") == "jocforge-implementer-supplement-seal.v1", "seal schema")
    require(seal.get("package_id") == PACKAGE_ID, "seal id")
    require(seal.get("manifest_sha256") == digest, "seal digest")
    require(seal.get("files") == files, "seal file list")

    evidence = read_json("EVIDENCE.json")
    require(evidence.get("status") == STATUS, "evidence status")
    require(evidence.get("implementation_ready") == "NO", "evidence readiness")
    mode = evidence["preferred_downmix_mode"]
    require(mode["host_to_native_xml_writer"] == "UNBOUND", "mode writer")
    require(mode["effective_stored_value"].startswith("UNBOUND"), "effective mode")
    require(mode["xml_default_literal"] == "loro", "mode schema default")
    require(mode["tokens_to_parser_u32"] == MODE_CODES, "mode code map")
    levels = evidence["mix_level_fields"]
    require(len(levels) == 4, "four level fields")
    require(all(field["default"] == {"token": "-3", "internal_code": 4} for field in levels), "level defaults")
    require(all(field["host_key"] != field["plugin_xml_key"] for field in levels), "host/native keys distinct")
    require(
        evidence["unbound_consumer_edges"]["mixlevel_u32_to_selected_same_owner_generation_read"] == "UNBOUND_NOT_CAPTURED",
        "owner-generation edge",
    )
    require(evidence["unbound_consumer_edges"]["first_matrix_consumer"] == "UNBOUND_NOT_TRACED", "matrix boundary")
    require(evidence["unbound_consumer_edges"]["first_bsi_consumer"] == "UNBOUND_NOT_TRACED", "BSI boundary")
    require(evidence["execution"]["dee_run"] is False, "DEE not run")

    vectors = read_json("INPUT-ONLY-VECTORS.json")
    require(vectors.get("status") == "INPUT_ONLY_NOT_RUN", "vector status")
    require(vectors["preferred_downmix_mode"] == [
        {"token": "loro", "parser_u32": 0, "xml_default_literal": True, "effective_stored_mode": "UNBOUND"},
        {"token": "ltrt", "parser_u32": 1, "xml_default_literal": False, "effective_stored_mode": "UNBOUND"},
        {"token": "ltrt-pl2", "parser_u32": 2, "xml_default_literal": False, "effective_stored_mode": "UNBOUND"},
        {"token": "not_indicated", "parser_u32": 3, "xml_default_literal": False, "effective_stored_mode": "UNBOUND"},
    ], "mode vector matrix")
    require(len(vectors["mix_level_fields"]) == 4, "level vector count")
    for item in vectors["mix_level_fields"]:
        require(item["default_token"] == "-3", "level vector default")
        require(item["token_to_internal_u32"] in (LEVEL_CODES, {k: v for k, v in LEVEL_CODES.items() if k not in {"+3", "+1.5", "0"}}), "level token map")
    sources = read_json("SOURCE-MANIFESTS.json")
    require(sources["source_package"]["checker_normal"] == "PASS", "source checker normal")
    require(sources["source_package"]["checker_python_optimized"] == "PASS", "source checker optimized")
    require(sum(bool(item["manifest_recomputed"]) for item in sources["reused_receipts"]) == 9, "recomputed parent count")
    require(sum(not item["manifest_recomputed"] for item in sources["reused_receipts"]) == 2, "unrecomputed parent count")
    report = (ROOT / "REPORT.md").read_text(encoding="utf-8")
    require("UNBOUND" in report and "NOT_RUN" in report, "open boundaries")
    print(f"PASS package={PACKAGE_ID}")
    print(f"PASS manifest_sha256={digest}")
    print("PASS downmix token maps and unresolved runtime/audio-consumer boundaries")


if __name__ == "__main__":
    main()
