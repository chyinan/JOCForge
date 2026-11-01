import hashlib
import json
import sys
from pathlib import Path


PACKAGE = Path(__file__).resolve().parent
PAYLOADS = {"CHECK.py", "EVIDENCE.json", "MINIMUM-RECEIPT-SCHEMA.json", "REPORT.md", "SOURCE-MANIFESTS.json", "WASH-VERIFICATION.json"}
PACKAGE_ID = "DEE-RUN145-RUN148-LOADONE-RECEIPT-BOUNDARY-V1"
INCOMING_MANIFEST = "6a0fd16292ed75b3e94b9ab945e41999bf1031cc32a40a5c3f9e8a715069ecf1"
INCOMING_SEAL = "3858ecda109f9a135cfe6d12fa2238b0c144e48606dc3480f711c9087025cd2e"
DDP_HASH = "3d66bcec36031fd48e6565d15f05fea656642377ca4f8c98cdce1cce8b7e95d2"


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
        "status": "HISTORICAL_LOADONE_RECEIPTS_MISSING; RUN145_RUN148_ORDINALS_UNBOUND; LOOSE_LOGS_UNBOUND",
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
    for run in (evidence["run145"], evidence["run148"]):
        require(run["absolute_manager_runner_ordinal"] == "UNBOUND"
                and run["loose_log_loader_markers"] == [], "historical ordinal/loose-log boundary mismatch")
        require("NOT_REFERENCED" in run["loose_log_binding"], "loose log improperly treated as run-bound")
    require(evidence["run145"]["sealed_receipt_loadone_order"] == "NOT_PRESENT"
            and evidence["run148"]["complete_ordered_audio_filter_set"] == "UNBOUND/NOT_CAPTURED",
            "sealed receipt coverage boundary mismatch")
    require(evidence["ddp_atmos_image_sha256"] == DDP_HASH
            and evidence["license_boundary"]["runtime_metadata_probe"] == "NOT_RUN"
            and evidence["license_boundary"]["license_contents_read_by_researcher"] is False,
            "target/license boundary mismatch")
    receipt = read_json("MINIMUM-RECEIPT-SCHEMA.json")
    require(receipt["status"] == "NOT_CAPTURED_FOR_RUN145_AND_RUN148"
            and "not a product api" in receipt["purpose"].lower(),
            "receipt request was presented as an existing observation or product API")
    sources = read_json("SOURCE-MANIFESTS.json")
    require(sources["incoming_package"]["manifest_sha256"] == INCOMING_MANIFEST
            and sources["incoming_package"]["detached_seal_sha256"] == INCOMING_SEAL
            and sources["incoming_package"]["source_hash_count"] == 47,
            "incoming package binding mismatch")
    verification = read_json("WASH-VERIFICATION.json")
    require(verification["incoming_manifest_sha256"] == INCOMING_MANIFEST
            and verification["incoming_seal_sha256"] == INCOMING_SEAL,
            "verification provenance mismatch")
    print(json.dumps({"status": "VALID_PARTIAL", "package_id": PACKAGE_ID,
                      "manifest_sha256": sha256(PACKAGE / "MANIFEST.json"),
                      "run145_ordinal": "UNBOUND", "run148_ordinal": "UNBOUND",
                      "source_hash_count": 47}, sort_keys=True))


if __name__ == "__main__":
    if sys.argv[1:] == ["--build-manifest"]:
        build_manifest()
    else:
        verify()
