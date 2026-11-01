import hashlib
import json
import sys
from pathlib import Path


PACKAGE = Path(__file__).resolve().parent
PAYLOADS = {"CHECK.py", "EVIDENCE.json", "REPORT.md", "SOURCE-MANIFESTS.json", "WASH-VERIFICATION.json"}
PACKAGE_ID = "R138-PROFILE-SEMANTIC-TEMPLATE-BOUNDARY-V1"
INCOMING = "d82300912609c9d1579c8ca265264bbfa26ee3f76c655f86fc78743d538e6991"
SEAL = "eb5189349cfc7461125c99ff38814c3c0aa1559b133fe329da57a7b15d787dcb"


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
        "status": "SEMANTIC_NONPATH_TEMPLATE_MATCH_BOUND; RUN_INSTANCE_PATHS_AND_MEDIA_IDENTITY_SEPARATE",
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
    require(evidence["r138_profile"]["normalized_semantic_sha256_redacting_only_file_name_and_path_text"]
            == evidence["first_love_template"]["normalized_nonpath_sha256"], "normalized config match")
    require(evidence["first_love_template"]["input_media_identity_equals_authorized_human_cat"] == "UNBOUND"
            and evidence["first_love_template"]["window_audio_classification"] == "QUIET", "template input boundary")
    require(evidence["other_profile_limits"]["local_xsd_bound_to_dee_executable"] is False
            and evidence["other_profile_limits"]["local_xsd_accepts_r138_element_order"] is False, "XSD scope")
    require(evidence["derived_job"]["created"] is False and evidence["derived_job"]["started"] is False,
            "no job created")
    sources = read_json("SOURCE-MANIFESTS.json")
    require(sources["incoming_package"]["manifest_sha256"] == INCOMING
            and sources["incoming_package"]["seal_sha256"] == SEAL, "source binding")
    verification = read_json("WASH-VERIFICATION.json")
    require(verification["incoming_manifest_sha256"] == INCOMING
            and verification["implementation_delivery"] == "REFERENCE_ONLY; NO_PRODUCT_RULE; implementation_ready=false",
            "verification scope")
    print(json.dumps({"status": "VALID_PARTIAL", "package_id": PACKAGE_ID,
                      "manifest_sha256": sha256(PACKAGE / "MANIFEST.json"),
                      "nonpath_semantics_match": True, "media_identity": "UNBOUND"}, sort_keys=True))


if __name__ == "__main__":
    if sys.argv[1:] == ["--build-manifest"]:
        build_manifest()
    else:
        verify()
