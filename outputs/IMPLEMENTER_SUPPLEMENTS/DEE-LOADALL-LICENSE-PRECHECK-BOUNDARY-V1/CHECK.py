import hashlib
import json
import sys
from pathlib import Path


PACKAGE = Path(__file__).resolve().parent
PAYLOADS = {"CHECK.py", "EVIDENCE.json", "REPORT.md", "SOURCE-MANIFESTS.json", "WASH-VERIFICATION.json"}
PACKAGE_ID = "DEE-LOADALL-LICENSE-PRECHECK-BOUNDARY-V1"
INCOMING_MANIFEST = "cef4d7b8e5d143b208bf50ab485b7b4408abcff41501098e9564ff766b38e621"
INCOMING_SEAL = "625dbd82f1ede034d853e4c5fbae430a55773be5df1d9a2ba96679e3859975c5"
HOST_SHA256 = "65485b6d35fa534b25109ab6f238971aaded73fc99c7f7fb119d1a5d5299234e"


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
        "status": "ENTRY_TO_LOADALL_STATICALLY_BOUND; LICENSE_CONTENT_PARSE_PRECONDITION; RUN145_RUN148_ORDINALS_UNBOUND",
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
    require(evidence["host_image_sha256"] == HOST_SHA256
            and evidence["license_access_scope"]["static_path_opens_and_parses_license_file_content_before_stage_runner"] is True
            and evidence["license_access_scope"]["actual_license_contents_read_by_researcher"] is False
            and evidence["license_access_scope"]["native_host_started"] is False,
            "license path/read boundary mismatch")
    require(evidence["static_call_chain"]["stage_runner_constructor_rva"] == "0x39230"
            and evidence["static_call_chain"]["load_all_rva"] == "0x169d10"
            and evidence["static_call_chain"]["job_config_lookup_order"].startswith("later"),
            "entry-to-LoadAll static chain mismatch")
    require(evidence["ordinal_boundary"]["run145_absolute_ordinal"] == "UNBOUND"
            and evidence["ordinal_boundary"]["run148_absolute_ordinal"] == "UNBOUND"
            and evidence["ordinal_boundary"]["run145_ordered_loadone_receipt"] == "NOT_CAPTURED"
            and evidence["ordinal_boundary"]["run148_ordered_loadone_receipt"] == "NOT_CAPTURED",
            "historical ordinal boundary mismatch")
    sources = read_json("SOURCE-MANIFESTS.json")
    require(sources["incoming_package"]["manifest_sha256"] == INCOMING_MANIFEST
            and sources["incoming_package"]["detached_seal_sha256"] == INCOMING_SEAL
            and sources["incoming_package"]["checker_normal"] == "VALID_PARTIAL"
            and sources["incoming_package"]["checker_python_optimized"] == "VALID_PARTIAL",
            "incoming source package binding mismatch")
    verification = read_json("WASH-VERIFICATION.json")
    require(verification["incoming_manifest_sha256"] == INCOMING_MANIFEST
            and verification["incoming_seal_sha256"] == INCOMING_SEAL,
            "Wash verification provenance mismatch")
    print(json.dumps({"status": "VALID_PARTIAL", "package_id": PACKAGE_ID,
                      "manifest_sha256": sha256(PACKAGE / "MANIFEST.json"),
                      "license_contents_read": False,
                      "run145_run148_ordinals": "UNBOUND"}, sort_keys=True))


if __name__ == "__main__":
    if sys.argv[1:] == ["--build-manifest"]:
        build_manifest()
    else:
        verify()
