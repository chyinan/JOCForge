import hashlib
import json
import sys
from pathlib import Path


PACKAGE = Path(__file__).resolve().parent
PAYLOADS = {"CHECK.py", "EVIDENCE.json", "INPUT-ONLY-VECTORS.json", "REPORT.md", "SOURCE-MANIFESTS.json", "WASH-VERIFICATION.json"}
PACKAGE_ID = "F12-DDPI-STREAM-A8-TO-CORE-STATE-BOUNDARY-V1"
INCOMING = "312cc1cddb20d8d27d492a66ebf030204ef0bce5853a2f8dabdf5557660cff25"


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
        "status": "STREAM_PLUS_A8_TO_CORE_PLUS_94_TRACED; SEMANTIC_AND_BSI_EDGE_UNBOUND",
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
    require(evidence["source_field"]["producer"] == "FUN_14034FF60 setparams case 0x15"
            and evidence["source_field"]["offset"] == "+0xA8", "source field")
    require(evidence["handoff"]["actual_source"] == "stream+0xA8"
            and evidence["handoff"]["destination"] == "codec_state+0x94"
            and evidence["handoff"]["value_rule"].startswith("source -1 => 0x33"), "direct handoff")
    require(evidence["separate_validation_field"]["typed_identity_join_to_codec_state_plus_0x94"] == "NOT_PROVEN"
            and evidence["bsi_writer"]["standard_header_u5_relationship"] == "UNBOUND", "field identity/wire boundary")
    vectors = read_json("INPUT-ONLY-VECTORS.json")["vectors"]
    expected_vectors = {"0x00000001":"0x00000001","0x00000000":"0x00000000","0xFFFFFFFF":"0x00000033"}
    require({v["input"]["stream_plus_a8_u32"]: v["expected"]["codec_state_plus_94_u32"] for v in vectors} == expected_vectors,
            "input-only copy vectors")
    require(evidence["policy"]["dee_run"] is False
            and evidence["policy"]["pcm_payload_bitstream_output_or_key_material_read"] is False
            and evidence["policy"]["implementation_delivery"] == "REFERENCE_ONLY; NO_PRODUCT_RULE", "policy")
    sources = read_json("SOURCE-MANIFESTS.json")
    require(sources["incoming_package"]["manifest_sha256"] == INCOMING
            and sources["incoming_package"]["manifest_seal_matches"] is True, "source binding")
    verification = read_json("WASH-VERIFICATION.json")
    require(verification["incoming_manifest_sha256"] == INCOMING
            and verification["implementation_delivery"] == "REFERENCE_ONLY; NO_PRODUCT_RULE; implementation_ready=false",
            "verification scope")
    print(json.dumps({"status":"VALID_PARTIAL","package_id":PACKAGE_ID,
                      "manifest_sha256":sha256(PACKAGE / "MANIFEST.json"),"id15":"UNBOUND","u5":"UNBOUND"},sort_keys=True))


if __name__ == "__main__":
    if sys.argv[1:] == ["--build-manifest"]:
        build_manifest()
    else:
        verify()
