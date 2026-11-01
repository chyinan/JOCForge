import hashlib
import json
import sys
from pathlib import Path


PACKAGE = Path(__file__).resolve().parent
PAYLOAD_FILES = {
    "CHECK.py",
    "CONTROLLED-VECTORS.json",
    "EVIDENCE.json",
    "REPORT.md",
    "SOURCE-IDENTITIES.json",
    "WASH-VERIFICATION.json",
}
PACKAGE_ID = "DEE-AUDIO-FILTER-ORDINAL-BOUNDARY-V1"
SOURCE_MANIFEST_SHA256 = "8c47327c3b2d37e730cab7066b001bb2aa3d00e68a4b314c15a004b29e706c85"
SOURCE_SEAL_SHA256 = "f658f9a432ba0424d8aa9d8eee098f0e1b4d4de2ca6fe6e201138b1781d81dbe"
DDP_MODULE_SHA256 = "3d66bcec36031fd48e6565d15f05fea656642377ca4f8c98cdce1cce8b7e95d2"


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


def model_ordinal(case):
    names = [row["module"] for row in case["module_path_order"]]
    require(names.count(case["selected_module"]) == 1, f"{case['id']}: selected module must occur once")
    selected_position = names.index(case["selected_module"])
    selected_count = case["module_path_order"][selected_position]["callback_count"]
    local_index = case["selected_local_index"]
    require(type(selected_count) is int and selected_count > 0, f"{case['id']}: invalid selected callback count")
    require(type(local_index) is int and 0 <= local_index < selected_count, f"{case['id']}: local index out of range")
    preceding = case["module_path_order"][:selected_position]
    require(all(type(row["callback_count"]) is int and row["callback_count"] >= 0 for row in preceding),
            f"{case['id']}: invalid preceding callback count")
    return sum(row["callback_count"] for row in preceding) + local_index


def build_manifest():
    actual = {path.name for path in PACKAGE.iterdir() if path.is_file()}
    expected = PAYLOAD_FILES
    require(actual == expected, f"package inventory mismatch: {sorted(actual ^ expected)}")
    payloads = []
    for name in sorted(PAYLOAD_FILES):
        path = PACKAGE / name
        payloads.append({"file": name, "bytes": path.stat().st_size, "sha256": sha256(path)})
    manifest = {
        "schema": "wash-only-research-manifest.v1",
        "package_id": PACKAGE_ID,
        "scope": "Hash-bound local callback order and manager-to-Runner relative ordinal relation; absolute global ordinal remains unbound.",
        "status": "SCOPED_RELATIVE_ORDER_BOUND; ABSOLUTE_GLOBAL_ORDINAL_UNBOUND",
        "implementation_ready": "NO",
        "payloads": payloads,
    }
    (PACKAGE / "MANIFEST.json").write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def verify():
    expected_files = PAYLOAD_FILES | {"MANIFEST.json"}
    actual_files = {path.name for path in PACKAGE.iterdir() if path.is_file()}
    require(actual_files == expected_files, f"package inventory mismatch: {sorted(actual_files ^ expected_files)}")
    require(all(path.is_file() for path in PACKAGE.iterdir()), "non-file item in package")

    manifest = read_json("MANIFEST.json")
    require(manifest["schema"] == "wash-only-research-manifest.v1", "manifest schema mismatch")
    require(manifest["package_id"] == PACKAGE_ID, "package id mismatch")
    require(manifest["implementation_ready"] == "NO", "readiness was incorrectly promoted")
    require(manifest["status"] == "SCOPED_RELATIVE_ORDER_BOUND; ABSOLUTE_GLOBAL_ORDINAL_UNBOUND",
            "status boundary changed")
    rows = {row["file"]: row for row in manifest["payloads"]}
    require(len(rows) == len(manifest["payloads"]) and set(rows) == PAYLOAD_FILES,
            "manifest payload inventory mismatch")
    for name, row in rows.items():
        path = PACKAGE / name
        require(path.stat().st_size == row["bytes"] and sha256(path) == row["sha256"],
                f"payload hash mismatch: {name}")

    evidence = read_json("EVIDENCE.json")
    require(evidence["implementation_ready"] == "NO", "evidence readiness mismatch")
    require(evidence["selected_module"]["sha256"] == DDP_MODULE_SHA256, "target module identity mismatch")
    require(evidence["selected_module"]["local_callback_count"] == 3, "local table size mismatch")
    require([row["local_index"] for row in evidence["selected_module"]["local_entries"]] == [0, 1, 2],
            "local slot order mismatch")
    require([row["callback_rva"] for row in evidence["selected_module"]["local_entries"]]
            == ["0x12a440", "0x12a4c0", "0x12a540"], "callback identities mismatch")
    require(evidence["selected_module"]["selected_route_local_index"] == 0
            and evidence["selected_module"]["selected_route_callback_rva"] == "0x12a440",
            "selected route local slot mismatch")
    require(evidence["ordering"]["global_ordinal_status"] == "ORDER_DEPENDENT/NOT_BOUND",
            "absolute ordinal was incorrectly bound")
    require(evidence["ordering"]["runner_absolute_index_status"]
            == "SAME_AS_GLOBAL_ORDINAL_IF_COMPLETE_LIST_IS_PRESENT; ACTUAL_VALUE_NOT_BOUND",
            "absolute Runner index boundary changed")
    require(evidence["run_crosscheck"]["local_slot_order_stability"]
            == "STATIC_STABLE_FOR_SAME_HASHED_DLL_IMAGE", "local stability claim changed")
    require(evidence["run_crosscheck"]["global_ordinal_stability"].startswith("NOT_ESTABLISHED;"),
            "cross-run global ordinal was incorrectly promoted")
    require(evidence["access_boundary"]["new_dee_or_windbg_run"] == "NOT_RUN"
            and evidence["access_boundary"]["raw_pcm_or_payload_read"] is False
            and evidence["access_boundary"]["private_key_or_license_contents_read"] is False,
            "access boundary changed")

    sources = read_json("SOURCE-IDENTITIES.json")
    incoming = sources["incoming_wash_package"]
    require(incoming["manifest_sha256"] == SOURCE_MANIFEST_SHA256
            and incoming["detached_seal_sha256"] == SOURCE_SEAL_SHA256
            and incoming["indexed_source_file_count"] == 27,
            "incoming package binding mismatch")
    require(incoming["source_checker_status_normal"] == "VALID_PARTIAL"
            and incoming["source_checker_status_optimized"] == "VALID_PARTIAL",
            "incoming checker status mismatch")
    parent = sources["parent_selected_callback_package"]
    require(parent["manifest_sha256"] == "00856107e8c19c232e7a560dd723e3cce804d6d2da8003144b43bf1f5484f25c",
            "parent package binding mismatch")

    vectors = read_json("CONTROLLED-VECTORS.json")
    require(vectors["evidence_class"] == "MODEL_ONLY_NOT_NATIVE_OBSERVATION", "vector evidence class mismatch")
    require(len(vectors["cases"]) == 4, "model vector count mismatch")
    for case in vectors["cases"]:
        expected = model_ordinal(case)
        require(case["expected_global_ordinal"] == expected, f"{case['id']}: global ordinal mismatch")
        require(case["expected_runner_ordinal"] == expected, f"{case['id']}: Runner ordinal mismatch")

    verification = read_json("WASH-VERIFICATION.json")
    require(verification["incoming_manifest_sha256"] == SOURCE_MANIFEST_SHA256
            and verification["incoming_seal_sha256"] == SOURCE_SEAL_SHA256,
            "verification provenance mismatch")
    require(verification["incoming_checker"]["normal"] == "PASS_VALID_PARTIAL"
            and verification["incoming_checker"]["python_optimized"] == "PASS_VALID_PARTIAL",
            "incoming checker result missing")
    require(verification["supplement_checker"]["normal"] == "PASS"
            and verification["supplement_checker"]["python_optimized"] == "PASS",
            "supplement checker result missing")

    print(json.dumps({
        "status": "VALID_PARTIAL",
        "package_id": PACKAGE_ID,
        "manifest_sha256": sha256(PACKAGE / "MANIFEST.json"),
        "incoming_manifest_sha256": SOURCE_MANIFEST_SHA256,
        "incoming_seal_sha256": SOURCE_SEAL_SHA256,
        "source_hash_count_verified": 27,
        "model_vectors_recomputed": len(vectors["cases"]),
        "absolute_global_ordinal": "ORDER_DEPENDENT/NOT_BOUND",
    }, sort_keys=True))


if __name__ == "__main__":
    if sys.argv[1:] == ["--build-manifest"]:
        build_manifest()
    else:
        verify()
