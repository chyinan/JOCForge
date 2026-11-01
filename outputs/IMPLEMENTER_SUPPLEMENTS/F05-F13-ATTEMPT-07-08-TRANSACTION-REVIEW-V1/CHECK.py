import hashlib
import json
import sys
from pathlib import Path


PACKAGE = Path(__file__).resolve().parent
PAYLOADS = {"CHECK.py", "EVIDENCE.json", "REPORT.md", "SOURCE-MANIFESTS.json", "WASH-VERIFICATION.json"}
PACKAGE_ID = "F05-F13-ATTEMPT-07-08-TRANSACTION-REVIEW-V1"
INCOMING = "7ae78144409d7836f8dad4da48ed1ef16aa2b2ce570063959689e2f0d0e0f06a"


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
        "status": "ATTEMPT08_SOURCE_AU_EOF_PASS_F15_POST_FAILURE; ATTEMPT07_MISMATCH_DETAILS_NOT_CAPTURED",
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
    require(evidence["attempt_07"]["terminal_status"] == "CALLBACK_INDEX_MISMATCH_BEFORE_EOF_NOT_DONE"
            and evidence["attempt_07"]["mismatch_callback_source_ordinals_tokens_generations_operands"] == "NOT_CAPTURED",
            "attempt-07 failure boundary")
    require(evidence["attempt_08"]["source_au_path"] == "PASS_WITHIN_LOGGED_SCOPE"
            and evidence["attempt_08"]["application_exit_code"] == 2
            and evidence["attempt_08"]["terminal_status"] == "F15_POST_FAILURE_NOT_DONE"
            and evidence["attempt_08"]["done_status"] == "NOT_REACHED", "attempt-08 terminal boundary")
    require(evidence["current_source_model"]["new_type_level_model"] == "MODEL_PROPOSAL_NOT_APPROVED"
            and evidence["current_source_model"]["attempt_08_causal_gate_attribution"] == "NOT_CAPTURED", "proposal scope")
    require(evidence["handling"]["product_artifact_opened"] is False
            and evidence["handling"]["encode_run_or_restart"] is False
            and evidence["handling"]["implementation_delivery"] == "REFERENCE_ONLY; NO_PRODUCT_RULE", "handling boundary")
    sources = read_json("SOURCE-MANIFESTS.json")
    require(sources["incoming_package"]["manifest_sha256"] == INCOMING, "source manifest binding")
    verification = read_json("WASH-VERIFICATION.json")
    require(verification["incoming_manifest_sha256"] == INCOMING
            and verification["implementation_delivery"] == "REFERENCE_ONLY; NO_PRODUCT_RULE; implementation_ready=false",
            "verification scope")
    print(json.dumps({"status": "VALID_PARTIAL", "package_id": PACKAGE_ID,
                      "manifest_sha256": sha256(PACKAGE / "MANIFEST.json"),
                      "attempt_07": "CALLBACK_INDEX_MISMATCH_NOT_CAPTURED", "attempt_08": "F15_POST_FAILURE"}, sort_keys=True))


if __name__ == "__main__":
    if sys.argv[1:] == ["--build-manifest"]:
        build_manifest()
    else:
        verify()
