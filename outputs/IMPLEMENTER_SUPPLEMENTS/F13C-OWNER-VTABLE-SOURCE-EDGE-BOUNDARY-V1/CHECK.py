import hashlib
import json
import sys
from pathlib import Path


PACKAGE = Path(__file__).resolve().parent
PAYLOADS = {"CHECK.py", "EVIDENCE.json", "REPORT.md", "SOURCE-MANIFESTS.json", "WASH-VERIFICATION.json"}
PACKAGE_ID = "F13C-OWNER-VTABLE-SOURCE-EDGE-BOUNDARY-V1"
INCOMING = "02d8a3bc76d1f25faba8a3ef5564c539f198834e618c6d14577b0f963bccb7b4"
PARENT_V6 = "b42be389f86010d6d4d62a7b726c550f0eaef2487ec57966c57fd8da25baa1ab"


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
        "status": "OWNER_SOURCE_AND_VTABLE_SLOT_BOUND; SELECTED_WRITER_UNBOUND",
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
    require(evidence["owner_input"]["requested_callsite"] == "0x1403c2eee"
            and evidence["owner_input"]["actual_containing_function"] == "FUN_1403c2e50"
            and evidence["owner_input"]["owner_expression"] == "*(param_1+0x8)", "owner source edge")
    require(evidence["other_vtable_method"]["function"] == "FUN_1403c3980"
            and evidence["other_vtable_method"]["slot_offset"] == "+0x8"
            and evidence["other_vtable_method"]["indirect_invocation_site"] == "UNRESOLVED", "vtable edge")
    for key in ["selected_ddp_joc_5_1_stage", "ddp_joc_ec3_writer", "f13c_writer"]:
        require(evidence["writer_reachability"][key] == "UNBOUND", f"writer edge {key}")
    require(evidence["execution_scope"]["encoder_launched"] is False
            and evidence["execution_scope"]["pcm_read"] is False
            and evidence["execution_scope"]["encoded_payload_read"] is False
            and evidence["execution_scope"]["private_license_hmac_key_content_read_or_exported"] is False,
            "execution boundary")
    sources = read_json("SOURCE-MANIFESTS.json")
    require(sources["incoming_package"]["manifest_sha256"] == INCOMING
            and sources["parent_source_package"]["manifest_sha256"] == PARENT_V6, "source chain")
    verification = read_json("WASH-VERIFICATION.json")
    require(verification["incoming_manifest_sha256"] == INCOMING
            and verification["implementation_delivery"] == "REFERENCE_ONLY; NO_PRODUCT_RULE; implementation_ready=false",
            "verification scope")
    print(json.dumps({"status": "VALID_PARTIAL", "package_id": PACKAGE_ID,
                      "manifest_sha256": sha256(PACKAGE / "MANIFEST.json"),
                      "owner_source": "BOUND", "writer": "UNBOUND"}, sort_keys=True))


if __name__ == "__main__":
    if sys.argv[1:] == ["--build-manifest"]:
        build_manifest()
    else:
        verify()
