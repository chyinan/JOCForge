import hashlib
import json
import sys
from pathlib import Path


PACKAGE = Path(__file__).resolve().parent
PAYLOADS = {"CHECK.py", "EVIDENCE.json", "REPORT.md", "SOURCE-MANIFESTS.json", "WASH-VERIFICATION.json"}
PACKAGE_ID = "F06-CUSTOM-BSI-VS-STANDARD-HEADER-PATH-V1"
INCOMING = "9681e89b9c9a36a65eb3fac7fb46ada75f26a2cf951ca0ef3e7a8d6671512caf"
PARENT_WASH = "6a3e74f2d7ac1763968e7c12228a56e8cd66e47ed97e34d279626e33900da9b5"
PARENT_SOURCE = "5d8af71b76ae1b0d56affbdbaa0679c143c31ff5e939727df657656507017647"


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
        "status": "CUSTOM_BSI_METADATA_VS_STANDARD_HEADER_DISTINCT; U16_TO_U5_MAPPING_UNBOUND",
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
    custom = evidence["custom_bsi_path"]
    require(custom["owner_field"]["offset"] == "+0x34"
            and custom["metadata_item"]["type"] == "0x0B"
            and custom["container_entries"] == 2, "custom BSI path")
    standard = evidence["standard_header_path"]
    require(standard["bit_offset"] == 45 and standard["width_bits"] == 5
            and standard["producer"] == "separate per-frame dialnorm parameter in encoder request", "standard header path")
    require(evidence["relation"]["same_structure"] is False
            and evidence["relation"]["same_serializer"] is False
            and evidence["relation"]["numeric_u16_to_u5_mapping"] == "UNBOUND/NOT_PROVEN", "path distinction")
    parent = evidence["parent_generation_observation"]
    require(parent["manifest_sha256"] == PARENT_SOURCE and parent["owner_u16_reread_in_that_full_run"] is False,
            "parent generation scope")
    require(evidence["policy"]["new_dee_run"] is False
            and evidence["policy"]["pcm_or_codec_payload_read"] is False
            and evidence["policy"]["implementation_delivery"] == "REFERENCE_ONLY; NO_PRODUCT_RULE", "policy")
    sources = read_json("SOURCE-MANIFESTS.json")
    require(sources["incoming_package"]["manifest_sha256"] == INCOMING
            and sources["parent_wash_packages"][0]["manifest_sha256"] == PARENT_WASH
            and sources["parent_source_package"]["manifest_sha256"] == PARENT_SOURCE, "source chain")
    incoming_payloads = {row["file"]: (row["bytes"], row["sha256"]) for row in sources["incoming_package"]["payloads"]}
    require(incoming_payloads == {
        "README.md": (2388, "8620e850ddbb1404a3cd55177a4cb165327b89be7a40be90afbf50ebe2c7b022"),
        "EVIDENCE.json": (4159, "78625b90e8241cb68385b5c67c7f40ccf299b5ea29bbbad7d670ef991dc275d6"),
        "MATRIX.json": (1916, "f736de5c1fe71290d43cbe6edb2a62753c43f7231baf12315bfd90a0c0c7d834"),
        "verify_bsi_custom_container_header_path.py": (5971, "408b2bf4444d03d34d048dc59f4e0e75c2b7364b8a8190bcbedfb6cda8d06880"),
    }, "incoming payload bindings")
    verification = read_json("WASH-VERIFICATION.json")
    require(verification["incoming_manifest_sha256"] == INCOMING
            and verification["implementation_delivery"] == "REFERENCE_ONLY; NO_PRODUCT_RULE; implementation_ready=false",
            "verification scope")
    print(json.dumps({"status": "VALID_PARTIAL", "package_id": PACKAGE_ID,
                      "manifest_sha256": sha256(PACKAGE / "MANIFEST.json"),
                      "custom_bsi_vs_header": "DISTINCT", "u16_to_u5": "UNBOUND"}, sort_keys=True))


if __name__ == "__main__":
    if sys.argv[1:] == ["--build-manifest"]:
        build_manifest()
    else:
        verify()
