import hashlib
import json
import sys
from pathlib import Path


PACKAGE = Path(__file__).resolve().parent
PAYLOADS = {"CHECK.py", "EVIDENCE.json", "REPORT.md", "SOURCE-MANIFESTS.json", "WASH-VERIFICATION.json"}
PACKAGE_ID = "PCM24-DDP-COMPOSITE-MASTER-READER-LINEAGE-ADDENDUM-V1"
INCOMING_MANIFEST = "4abd471f5d943eec67e2c1d88548d3a3009dd782537af90bc64e416665703472"
ERRATUM_MANIFEST = "f1e303eb14ebf6153c29e11cba4f3ed5cf03df76ca13ec852ad68cbd4a1c5e89"


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
        "status": "DDP_INTERNAL_RECORD_TO_COMPOSITE_MASTER_PATH_BOUND; HOST_ADM_OWNER_GENERATION_UNBOUND",
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
    require(evidence["status"].startswith("DDP_RECORD_TO_COMPOSITE_MASTER_GETTER_AND_DESCRIPTOR_PATH_BOUND;"),
            "DDP-internal evidence status mismatch")
    require(evidence["owner_plus_0x350"]["classification"] == "OUTPUT_FILE_LIST_NOT_SOURCE_ARRAY",
            "+0x350 correction mismatch")
    path = evidence["composite_master_path"]
    require(path["factory_output_argument"] == "composition_state+0x50"
            and path["getter"] == "FUN_18032F1D0 returns composition_state+0x50"
            and path["descriptor_consumer"] == "FUN_180186870"
            and path["dispatch_slot_byte_offset"] == "0xC8"
            and path["static_typed_ddp_internal_path"] == "BOUND",
            "composite-master getter/dispatch path mismatch")
    require(evidence["unbound"]["host_selected_adm_master_to_ddp_composite_master"] is True
            and evidence["unbound"]["host_selected_adm_generation_to_ddp_record_generation"] is True,
            "host owner/generation boundary was promoted")
    require(evidence["run152_run153_provenance"]["input_payload_identity"] == "UNBOUND/NO; payload was not rehashed",
            "Run152/153 input identity scope mismatch")
    require(evidence["access_boundary"]["new_encoder_run"] == "NOT_RUN"
            and evidence["access_boundary"]["bitstream_read_or_parsed"] is False
            and evidence["access_boundary"]["audio_or_joc_playback"] == "NOT_RUN"
            and evidence["access_boundary"]["row_contents_or_pcm_read"] is False,
            "run/media access boundary mismatch")
    sources = read_json("SOURCE-MANIFESTS.json")
    require(sources["incoming_package"]["manifest_sha256"] == INCOMING_MANIFEST
            and sources["incoming_package"]["checker_normal"] == "PASS"
            and sources["incoming_package"]["checker_python_optimized"] == "PASS"
            and sources["provenance_parent"]["manifest_sha256"] == ERRATUM_MANIFEST,
            "incoming provenance binding mismatch")
    verification = read_json("WASH-VERIFICATION.json")
    require(verification["incoming_manifest_sha256"] == INCOMING_MANIFEST,
            "verification provenance mismatch")
    print(json.dumps({"status": "VALID_PARTIAL", "package_id": PACKAGE_ID,
                      "manifest_sha256": sha256(PACKAGE / "MANIFEST.json"),
                      "ddp_internal_path": "BOUND", "host_owner_generation": "UNBOUND"}, sort_keys=True))


if __name__ == "__main__":
    if sys.argv[1:] == ["--build-manifest"]:
        build_manifest()
    else:
        verify()
