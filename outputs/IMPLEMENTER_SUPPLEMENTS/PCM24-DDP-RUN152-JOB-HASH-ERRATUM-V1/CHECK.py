import hashlib
import json
import sys
from pathlib import Path


PACKAGE = Path(__file__).resolve().parent
PAYLOADS = {"CHECK.py", "EVIDENCE.json", "REPORT.md", "SOURCE-MANIFESTS.json", "WASH-VERIFICATION.json"}
PACKAGE_ID = "PCM24-DDP-RUN152-JOB-HASH-ERRATUM-V1"
BASE_MANIFEST = "1082dd1460fa133a727ff0daeed99b18bfa2b8e6668bdfc4f3f2a2876153baef"
ERRATUM_MANIFEST = "f1e303eb14ebf6153c29e11cba4f3ed5cf03df76ca13ec852ad68cbd4a1c5e89"
RUN152_XML = "31e642109943bc9a93a1704b8a5c6137eb6a50dfaa4e782f0b5dc2c897a8dd9c"
RUN153_XML = "c1440b3ab909ec59574fb3bede15bc047e9b61428162e8cbb2c9d63d4c459cf6"


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
    actual = {path.name for path in PACKAGE.iterdir() if path.is_file()}
    require(actual == PAYLOADS, f"payload inventory mismatch: {sorted(actual ^ PAYLOADS)}")
    files = [{"file": name, "bytes": (PACKAGE / name).stat().st_size, "sha256": sha256(PACKAGE / name)}
             for name in sorted(PAYLOADS)]
    manifest = {
        "schema": "jocforge-washed-implementer-supplement.v1",
        "package_id": PACKAGE_ID,
        "status": "RUN152_JOB_HASH_CORRECTED; INPUT_IDENTITY_UNBOUND",
        "implementation_ready": False,
        "source_reverse_manifest_sha256": ERRATUM_MANIFEST,
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
            and manifest["source_reverse_manifest_sha256"] == ERRATUM_MANIFEST,
            "manifest identity/readiness mismatch")
    rows = {row["file"]: row for row in manifest["files"]}
    require(len(rows) == len(manifest["files"]) and set(rows) == PAYLOADS, "manifest file inventory mismatch")
    for name, row in rows.items():
        path = PACKAGE / name
        require(path.stat().st_size == row["bytes"] and sha256(path) == row["sha256"], f"payload hash mismatch: {name}")
    evidence = read_json("EVIDENCE.json")
    require(evidence["correction"]["run152_job_xml_sha256_correct"] == RUN152_XML
            and evidence["correction"]["run152_old_misrecorded_hash"] == RUN153_XML
            and evidence["correction"]["run153_job_xml_sha256"] == RUN153_XML,
            "Run152/Run153 job hash mapping mismatch")
    require(evidence["correction"]["run152_job_xml_sha256_correct"] != evidence["correction"]["run152_old_misrecorded_hash"],
            "Run152 correction did not change the erroneous hash")
    require(evidence["unchanged_observation_scope"]["sample_identity"] == "UNBOUND/NO"
            and evidence["unchanged_observation_scope"]["bitstream_read_or_parsed"] is False
            and evidence["unchanged_observation_scope"]["playback"] == "NOT_RUN",
            "Run152/153 identity or output boundary was promoted")
    sources = read_json("SOURCE-MANIFESTS.json")
    require(sources["base_package"]["manifest_sha256"] == BASE_MANIFEST
            and sources["erratum_package"]["manifest_sha256"] == ERRATUM_MANIFEST
            and sources["base_package"]["checker_normal"] == "PASS"
            and sources["base_package"]["checker_python_optimized"] == "PASS"
            and sources["erratum_package"]["checker_normal"] == "PASS"
            and sources["erratum_package"]["checker_python_optimized"] == "PASS",
            "incoming package bindings/checker states mismatch")
    verification = read_json("WASH-VERIFICATION.json")
    require(verification["base_manifest_sha256"] == BASE_MANIFEST
            and verification["erratum_manifest_sha256"] == ERRATUM_MANIFEST,
            "Wash verification provenance mismatch")
    print(json.dumps({"status": "VALID_CORRECTION", "package_id": PACKAGE_ID,
                      "manifest_sha256": sha256(PACKAGE / "MANIFEST.json"),
                      "run152_job_xml_sha256": RUN152_XML,
                      "run153_job_xml_sha256": RUN153_XML,
                      "sample_identity": "UNBOUND/NO"}, sort_keys=True))


if __name__ == "__main__":
    if sys.argv[1:] == ["--build-manifest"]:
        build_manifest()
    else:
        verify()
