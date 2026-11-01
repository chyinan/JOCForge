import hashlib
import json
import sys
from pathlib import Path


PACKAGE = Path(__file__).resolve().parent
PAYLOADS = {
    "CHECK.py",
    "EVIDENCE.json",
    "INPUT-ONLY-VECTORS.json",
    "REPORT.md",
    "SOURCE-MANIFESTS.json",
    "WASH-VERIFICATION.json",
}
PACKAGE_ID = "DEE-AUDIO-FILTER-SEARCH-PATH-PROVENANCE-ADDENDUM-V1"
INCOMING_MANIFEST = "97184837a407a40e7caa41a22ed2a6d386f59eb503b367061ce99a0c5e78bea3"
INCOMING_SEAL = "541a7fa756398398c11f1ae69b54f0ed8e224c9bedcd9cb37d52c5b5b5790cf6"
HOST_SHA256 = "65485b6d35fa534b25109ab6f238971aaded73fc99c7f7fb119d1a5d5299234e"
DDP_SHA256 = "3d66bcec36031fd48e6565d15f05fea656642377ca4f8c98cdce1cce8b7e95d2"


def require(condition, message):
    if not condition:
        raise RuntimeError(message)


def sha256(path):
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def read_json(name):
    return json.loads((PACKAGE / name).read_text(encoding="utf-8"))


def selected_path(environment_value, fallback_value):
    return environment_value if environment_value else fallback_value


def build_manifest():
    actual = {path.name for path in PACKAGE.iterdir() if path.is_file()} - {"MANIFEST.json"}
    require(actual == PAYLOADS, f"payload inventory mismatch: {sorted(actual ^ PAYLOADS)}")
    files = [{"file": name, "bytes": (PACKAGE / name).stat().st_size, "sha256": sha256(PACKAGE / name)}
             for name in sorted(PAYLOADS)]
    manifest = {
        "schema": "jocforge-washed-implementer-supplement.v1",
        "package_id": PACKAGE_ID,
        "status": "PLUGIN_PATH_PRODUCER_STATICALLY_BOUND; RUN_EFFECTIVE_LOADONE_ORDER_UNBOUND; ABSOLUTE_ORDINAL_UNBOUND",
        "implementation_ready": False,
        "source_reverse_manifest_sha256": INCOMING_MANIFEST,
        "files": files,
    }
    (PACKAGE / "MANIFEST.json").write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def verify():
    expected = PAYLOADS | {"MANIFEST.json"}
    actual = {path.name for path in PACKAGE.iterdir() if path.is_file()}
    require(actual == expected, f"file inventory mismatch: {sorted(actual ^ expected)}")
    manifest = read_json("MANIFEST.json")
    require(manifest["schema"] == "jocforge-washed-implementer-supplement.v1"
            and manifest["package_id"] == PACKAGE_ID
            and manifest["implementation_ready"] is False
            and manifest["source_reverse_manifest_sha256"] == INCOMING_MANIFEST,
            "manifest identity/readiness mismatch")
    rows = {row["file"]: row for row in manifest["files"]}
    require(len(rows) == len(manifest["files"]) and set(rows) == PAYLOADS, "manifest payload inventory mismatch")
    for name, row in rows.items():
        path = PACKAGE / name
        require(path.stat().st_size == row["bytes"] and sha256(path) == row["sha256"],
                f"payload hash mismatch: {name}")

    evidence = read_json("EVIDENCE.json")
    require(evidence["status"] == "PLUGIN_PATH_PRODUCER_STATICALLY_BOUND; RUN_EFFECTIVE_LOADONE_ORDER_UNBOUND; ABSOLUTE_ORDINAL_UNBOUND",
            "evidence status mismatch")
    require(evidence["host_image_sha256"] == HOST_SHA256
            and evidence["ordinal_relation"]["ddp_atmos_module_sha256"] == DDP_SHA256,
            "host or selected module identity mismatch")
    require(evidence["path_producer"]["environment_override"] == "DEE_PLUGIN_PATH"
            and evidence["path_producer"]["fallback_config_key"] == "dee_loader.list"
            and evidence["path_producer"]["load_all_function_rva"] == "0x169d10",
            "plugin path producer boundary mismatch")
    require(evidence["ordinal_relation"]["run145_absolute_ordinal"] == "UNBOUND"
            and evidence["ordinal_relation"]["run148_absolute_ordinal"] == "UNBOUND",
            "absolute ordinal was incorrectly bound")
    require(evidence["access_boundary"]["runtime_metadata_capture"] == "NOT_RUN"
            and evidence["access_boundary"]["encode_run"] == "NOT_RUN"
            and evidence["access_boundary"]["media_pcm_payload_bitstream_or_key_read"] is False,
            "runtime/data access boundary changed")

    vectors = read_json("INPUT-ONLY-VECTORS.json")
    require(vectors["evidence_class"] == "SYNTHETIC_STATIC_CONTROL_FLOW_ONLY" and len(vectors["cases"]) == 2,
            "vector scope/count mismatch")
    for case in vectors["cases"]:
        inp = case["input"]
        expected_path = selected_path(inp["dee_plugin_path"], inp["dee_loader_list"])
        require(expected_path == case["expected_selected_path_string"], f"path precedence mismatch: {case['id']}")
        require(case["expected_loadone_module_order"] == "NOT_SUPPLIED", f"runtime module order inferred: {case['id']}")

    sources = read_json("SOURCE-MANIFESTS.json")
    incoming = sources["incoming_package"]
    require(incoming["manifest_sha256"] == INCOMING_MANIFEST
            and incoming["detached_seal_sha256"] == INCOMING_SEAL
            and incoming["source_hash_count"] == 29
            and incoming["checker_normal"] == "VALID_PARTIAL"
            and incoming["checker_python_optimized"] == "VALID_PARTIAL",
            "incoming package binding mismatch")
    verification = read_json("WASH-VERIFICATION.json")
    require(verification["incoming_manifest_sha256"] == INCOMING_MANIFEST
            and verification["incoming_seal_sha256"] == INCOMING_SEAL,
            "Wash verification provenance mismatch")

    print(json.dumps({"status": "VALID_PARTIAL", "package_id": PACKAGE_ID,
                      "manifest_sha256": sha256(PACKAGE / "MANIFEST.json"),
                      "input_vectors_recomputed": len(vectors["cases"]),
                      "run145_ordinal": "UNBOUND", "run148_ordinal": "UNBOUND"}, sort_keys=True))


if __name__ == "__main__":
    if sys.argv[1:] == ["--build-manifest"]:
        build_manifest()
    else:
        verify()
