import hashlib
import json
import sys
from pathlib import Path


PACKAGE = Path(__file__).resolve().parent
PAYLOADS = {"CAPTURE-PLAN.json", "CHECK.py", "EVIDENCE.json", "REPORT.md", "SOURCE-MANIFESTS.json", "WASH-VERIFICATION.json"}
PACKAGE_ID = "F13C-READER-METADATA-AND-SETUP-POINTER-LIFETIME-V2"
INCOMING_MANIFEST = "164409eeda49b71e52a9800feaa25c91aae0f45580f258c7804e5533c1de9d9"
WASH_V1_MANIFEST = "d257cfab334addceacf512dc27165989001c343628398ab3f4d95ec114d6ac09"


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
        "status": "SCHEMA_METADATA_CALLBACKS_OBSERVED; R138_SELECTED_READER_UNBOUND; E_SETUP_TRANSIENT_ALIAS_AND_FINAL_COPY_BOUND",
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
    require(evidence["schema_only_host_observation"]["route_selected"] is False
            and evidence["schema_only_host_observation"]["selected_r138_audio_input_reader"] == "NOT_OBSERVED",
            "schema-mode selection boundary mismatch")
    lifetime = evidence["setup_pointer_lifetime"]
    require(lifetime["temporary_assignment"] == "this+0x50 temporarily receives param4[2]"
            and lifetime["post_allocation_pointer_relation"].startswith("setup-owned table is distinct"),
            "setup transient/final pointer lifetime mismatch")
    require(evidence["converter_image_identity"]["current_e_drive_converter_sha256"]
            != evidence["converter_image_identity"]["older_d_drive_converter_sha256"],
            "converter image identity distinction missing")
    require(evidence["unbound"]["R138_audio_adm_reader_factory"] is True
            and evidence["unbound"]["DDP_packet_to_selected_reader_generation"] is True,
            "reader/DDP identity was incorrectly bound")
    require(evidence["access_boundary"]["runtime_r138_observation"] == "NOT_RUN"
            and evidence["access_boundary"]["pcm_payload_or_row_table_bytes_read"] is False
            and evidence["access_boundary"]["key_or_license_contents_read"] is False,
            "runtime/data boundary mismatch")
    plan = read_json("CAPTURE-PLAN.json")
    require(plan["status"] == "NOT_RUN; SPECIFICATION_ONLY"
            and plan["host_reader_stop"]["stop_before_Run"] is True,
            "observation plan was promoted to a runtime capture")
    sources = read_json("SOURCE-MANIFESTS.json")
    require(sources["incoming_package"]["manifest_sha256"] == INCOMING_MANIFEST
            and sources["incoming_package"]["checker_normal"] == "PASS"
            and sources["incoming_package"]["checker_python_optimized"] == "PASS",
            "incoming package provenance mismatch")
    require(sources["supersedes_source_binding_of_wash_v1"]["manifest_sha256"] == WASH_V1_MANIFEST,
            "superseded Wash v1 binding mismatch")
    print(json.dumps({"status": "VALID_PARTIAL", "package_id": PACKAGE_ID,
                      "manifest_sha256": sha256(PACKAGE / "MANIFEST.json"),
                      "R138_reader": "UNBOUND", "setup_pointer_lifetime": "STATICALLY_BOUND"}, sort_keys=True))


if __name__ == "__main__":
    if sys.argv[1:] == ["--build-manifest"]:
        build_manifest()
    else:
        verify()
