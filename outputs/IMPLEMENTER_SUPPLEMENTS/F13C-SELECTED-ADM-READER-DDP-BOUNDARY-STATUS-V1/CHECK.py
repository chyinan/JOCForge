import hashlib
import json
import sys
from pathlib import Path


PACKAGE = Path(__file__).resolve().parent
PAYLOADS = {
    "CHECK.py",
    "EVIDENCE.json",
    "OBSERVATION-PLAN.json",
    "REPORT.md",
    "SOURCE-MANIFESTS.json",
    "WASH-VERIFICATION.json",
}
PACKAGE_ID = "F13C-SELECTED-ADM-READER-DDP-BOUNDARY-STATUS-V1"
INCOMING_MANIFEST = "33cc65b97deec34c97990a514b96c89d5880c93e4f3f8a8e9d972014f373a1f5"


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


def build_manifest():
    actual = {path.name for path in PACKAGE.iterdir() if path.is_file()} - {"MANIFEST.json"}
    require(actual == PAYLOADS, f"payload inventory mismatch: {sorted(actual ^ PAYLOADS)}")
    files = [{"file": name, "bytes": (PACKAGE / name).stat().st_size, "sha256": sha256(PACKAGE / name)}
             for name in sorted(PAYLOADS)]
    manifest = {
        "schema": "jocforge-washed-implementer-supplement.v1",
        "package_id": PACKAGE_ID,
        "status": "SCOPED_STATIC_ROUTE_AUDIT_COMPLETE; SELECTED_ADM_READER_TO_DDP_ROW_UNBOUND",
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
    require(evidence["static_route_audit"]["audio_adm_direct_reference_count_in_scoped_host_trace"] == 1
            and evidence["static_route_audit"]["direct_reference_function"] == "FUN_140035700"
            and evidence["static_route_audit"]["selected_r138_audio_input_factory"] == "UNBOUND",
            "selected reader scope mismatch")
    require(evidence["edge_matrix"][2]["status"]
            == "STATIC_DATA_LINEAGE_PROVEN; DISTINCT_POINTERS; RUNTIME_GENERATION_UNBOUND"
            and evidence["edge_matrix"][6]["status"]
            == "BOUND_WITHIN_ONE_PRIOR_CAPTURE; NOT_JOINED_TO_SELECTED_READER",
            "edge matrix boundary mismatch")
    require(evidence["runtime_boundary"]["capture_in_incoming_addendum"] == "NOT_RUN"
            and evidence["runtime_boundary"]["dee_job_started"] is False
            and evidence["runtime_boundary"]["bitstream_emission"] == "NOT_RUN",
            "runtime status mismatch")

    plan = read_json("OBSERVATION-PLAN.json")
    require(plan["execution_status"] == "NOT_RUN; SPECIFICATION_ONLY"
            and len(plan["capture_points"]) == 7
            and "unbound" in plan["one_attempt_stop"].lower()
            and "unbound" in plan["activation_identity"]["missing_generation_rule"].lower(),
            "metadata plan was misrepresented as an observation")
    sources = read_json("SOURCE-MANIFESTS.json")
    require(sources["incoming_package"]["manifest_sha256"] == INCOMING_MANIFEST
            and sources["incoming_package"]["checker_normal"] == "PASS"
            and sources["incoming_package"]["checker_python_optimized"] == "PASS",
            "incoming package provenance mismatch")
    verification = read_json("WASH-VERIFICATION.json")
    require(verification["incoming_manifest_sha256"] == INCOMING_MANIFEST,
            "Wash verification provenance mismatch")

    print(json.dumps({"status": "VALID_PARTIAL", "package_id": PACKAGE_ID,
                      "manifest_sha256": sha256(PACKAGE / "MANIFEST.json"),
                      "selected_reader": "UNBOUND", "packet_to_normalizer": "BOUND_WITHIN_PRIOR_CAPTURE_ONLY"}, sort_keys=True))


if __name__ == "__main__":
    if sys.argv[1:] == ["--build-manifest"]:
        build_manifest()
    else:
        verify()
