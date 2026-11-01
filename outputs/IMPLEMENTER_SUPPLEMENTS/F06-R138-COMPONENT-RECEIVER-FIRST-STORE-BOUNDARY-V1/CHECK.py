from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "F06-R138-COMPONENT-RECEIVER-FIRST-STORE-BOUNDARY-V1"
PAYLOADS = ("EVIDENCE.json", "INPUT-ONLY-VECTORS.json", "REPORT.md", "SOURCE-MANIFESTS.json")
SOURCE_MANIFEST = "5705697e89bd314346637742db1151ffc84062449e26a78163600d3522f1ac37"
SCAN_PARENT = "44cb132c8623693e89cb612c59d42479cf38708a7a81f0f438b231c29ab905df"
MODE_PARENT = "ba95fbedd155591bddf10eb11a7b21f3dc3c5dab414b52627c3a1e5c8dcbf74c"
TARGET_IMAGE = "3d66bcec36031fd48e6565d15f05fea656642377ca4f8c98cdce1cce8b7e95d2"
JOB_SHA256 = "fa65143f0bd7de52b36367137e61d681baa4c17824aded10be26317e7d1018a4"
ADM_SHA256 = "7826dcbef37ef69642a6164c97a37bcfb3f917cea74aa0f0a30d76d0a825f1d1"


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def load_json(name: str) -> dict:
    return json.loads((ROOT / name).read_text(encoding="utf-8"))


def build_manifest() -> None:
    files = []
    for name in PAYLOADS:
        path = ROOT / name
        require(path.is_file(), f"missing payload: {name}")
        files.append({"file": name, "bytes": path.stat().st_size, "sha256": sha256(path)})
    manifest = {
        "schema": "jocforge-washed-implementer-supplement.v1",
        "package_id": PACKAGE_ID,
        "source_reverse_manifest_sha256": SOURCE_MANIFEST,
        "status": "RECEIVER_FIRST_STORE_BOUND; R138_STATE_SOURCE_UNBOUND",
        "implementation_ready": False,
        "files": files,
    }
    raw = json.dumps(manifest, ensure_ascii=False, indent=2) + "\n"
    (ROOT / "MANIFEST.json").write_text(raw, encoding="utf-8", newline="\n")
    (ROOT / "MANIFEST.sha256").write_text(f"{sha256(ROOT / 'MANIFEST.json')}  MANIFEST.json\n", encoding="ascii", newline="\n")


def check() -> None:
    manifest = load_json("MANIFEST.json")
    require(manifest.get("package_id") == PACKAGE_ID, "package id mismatch")
    require(manifest.get("source_reverse_manifest_sha256") == SOURCE_MANIFEST, "source mismatch")
    require(manifest.get("status") == "RECEIVER_FIRST_STORE_BOUND; R138_STATE_SOURCE_UNBOUND", "status mismatch")
    require(manifest.get("implementation_ready") is False, "must not claim readiness")
    entries = manifest.get("files")
    require(isinstance(entries, list) and {entry.get("file") for entry in entries} == set(PAYLOADS), "payload list mismatch")
    for entry in entries:
        path = ROOT / entry["file"]
        require(path.is_file(), f"missing payload: {entry['file']}")
        require(path.stat().st_size == entry.get("bytes"), f"size mismatch: {entry['file']}")
        require(sha256(path) == entry.get("sha256"), f"hash mismatch: {entry['file']}")
    seal = (ROOT / "MANIFEST.sha256").read_text(encoding="ascii").strip()
    require(seal == f"{sha256(ROOT / 'MANIFEST.json')}  MANIFEST.json", "manifest seal mismatch")

    evidence = load_json("EVIDENCE.json")
    require(evidence["source"]["manifest_sha256"] == SOURCE_MANIFEST, "evidence source mismatch")
    require(evidence["source"]["parent_plus0x54_scan_manifest_sha256"] == SCAN_PARENT, "scan parent mismatch")
    require(evidence["parents"][0]["manifest_sha256"] == "345d0b434b0170d18fcd1046a414fed6dc973256af8144eb33f976ec76182a0b", "Wash parent mismatch")
    require(evidence["target"]["image_sha256"] == TARGET_IMAGE, "target image mismatch")
    require(evidence["target"]["r138_job_xml_sha256"] == JOB_SHA256, "job identity mismatch")
    require(evidence["target"]["input_adm_sha256"] == ADM_SHA256, "input identity mismatch")
    require(evidence["typed_chain"]["receiver_first_store_plus_0x54"] == "BOUND_BY_TYPED_BLOCK_COPY", "receiver store mismatch")
    require(evidence["remaining_origin"]["process_state_plus_0x7e4_to_r138_job_decoder"] == "UNBOUND", "state source must remain UNBOUND")
    require(evidence["remaining_origin"]["custom_bsi_zero_applicability"] == "UNBOUND", "zero applicability must remain UNBOUND")
    require(evidence["execution_scope"]["implementation_ready"] is False, "must remain not ready")

    vectors = load_json("INPUT-ONLY-VECTORS.json")
    require(vectors["vectors"][0]["expected"]["member_byte_range_in_copy"] == "4..7", "copy overlap mismatch")
    require(vectors["vectors"][1]["expected"]["process_state_to_r138_job_decoder"] == "UNBOUND", "vector origin mismatch")
    require(vectors["lineage"]["source_manifest_sha256"] == SOURCE_MANIFEST, "vector source mismatch")
    sources = load_json("SOURCE-MANIFESTS.json")
    require(sources["incoming_package"]["manifest_sha256"] == SOURCE_MANIFEST, "source binding mismatch")
    require(sources["previous_scan"]["manifest_sha256"] == SCAN_PARENT, "scan reference mismatch")
    require(sources["target"]["input_adm_sha256"] == ADM_SHA256, "source input mismatch")
    require(sources["raw_trace_script_or_media_copied"] is False, "raw material must not be copied")
    print(f"PASS package={PACKAGE_ID} receiver_first_store=BOUND R138_state_origin=UNBOUND")


if __name__ == "__main__":
    if sys.argv[1:] == ["--build-manifest"]:
        build_manifest()
    elif not sys.argv[1:]:
        check()
    else:
        raise SystemExit("usage: python CHECK.py [--build-manifest]")
