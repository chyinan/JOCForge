import hashlib
import json
import sys
from pathlib import Path


PACKAGE = Path(__file__).resolve().parent
PAYLOADS = {"CHECK.py", "EVIDENCE.json", "REPORT.md", "SOURCE-MANIFESTS.json", "WASH-VERIFICATION.json"}
PACKAGE_ID = "R138-AUDIO-INPUT-TAG-SELECTION-BOUNDARY-V1"
INCOMING = "80d2e2c64880c0a38483b51cd51094f1092fda52936cfb28b60513184b256836"
SEAL = "5b5e69bf4e9eeb5b30c0e3f6c548d0967dd1d9f717eeeea5745692d85170e34f"


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
    files = [
        {"file": name, "bytes": (PACKAGE / name).stat().st_size, "sha256": sha256(PACKAGE / name)}
        for name in sorted(PAYLOADS)
    ]
    manifest = {
        "schema": "jocforge-washed-implementer-supplement.v1",
        "package_id": PACKAGE_ID,
        "status": "EXACT_XML_TAG_MATCH_RULE_STATICALLY_BOUND; R138_SELECTED_INDEX_UNBOUND",
        "implementation_ready": False,
        "source_reverse_manifest_sha256": INCOMING,
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
            and manifest["source_reverse_manifest_sha256"] == INCOMING, "manifest identity/readiness")
    rows = {row["file"]: row for row in manifest["files"]}
    require(len(rows) == len(manifest["files"]) and set(rows) == PAYLOADS, "manifest payload inventory")
    for name, row in rows.items():
        path = PACKAGE / name
        require(path.stat().st_size == row["bytes"] and sha256(path) == row["sha256"], f"payload hash: {name}")
    evidence = read_json("EVIDENCE.json")
    require(evidence["package_id"] == PACKAGE_ID and evidence["implementation_ready"] is False, "evidence identity")
    require(evidence["profile_receipt"]["input_type"] == "audio.adm"
            and evidence["profile_receipt"]["output_profile"] == "encode_to_atmos_ddp", "input/output profile separation")
    selector = evidence["static_paths"]["xml_selector"]
    require(selector["tag_storage"] == "StageObjectWithStorage+0x210"
            and selector["candidate_vector"] == "+0xB0..+0xB8"
            and selector["comparison"] == "equal byte length and byte-for-byte equality"
            and selector["duplicate_name_behavior"] == "last equal-name entry wins", "selector behavior")
    require(evidence["unbound"]["selector_vector_to_stage_runner_vector_identity"] == "UNBOUND"
            and evidence["unbound"]["selected_stage_runner_input_candidate_index"] == "UNBOUND/NO", "selection identity")
    require(evidence["minimum_next_observation"]["capture_status"] == "NOT_RUN"
            and evidence["minimum_next_observation"]["authorization_from_this_package"] is False, "observation status")
    sources = read_json("SOURCE-MANIFESTS.json")
    require(sources["incoming_package"]["manifest_sha256"] == INCOMING
            and sources["incoming_package"]["seal_sha256"] == SEAL, "source binding")
    verification = read_json("WASH-VERIFICATION.json")
    require(verification["incoming_manifest_sha256"] == INCOMING
            and verification["incoming_seal_sha256"] == SEAL
            and verification["implementation_delivery"] == "REFERENCE_ONLY; NO_PRODUCT_RULE; implementation_ready=false",
            "verification scope")
    print(json.dumps({"status": "VALID_PARTIAL", "package_id": PACKAGE_ID,
                      "manifest_sha256": sha256(PACKAGE / "MANIFEST.json"),
                      "selection_rule": "EXACT_XML_TAG_MATCH_LAST_EQUAL_NAME_WINS",
                      "r138_index": "UNBOUND/NO"}, sort_keys=True))


if __name__ == "__main__":
    if sys.argv[1:] == ["--build-manifest"]:
        build_manifest()
    else:
        verify()
