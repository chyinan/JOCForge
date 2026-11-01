from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "F06ND-CATEGORY5-SELECTED-PROCESS-BOUNDARY-V1"
PAYLOADS = ("EVIDENCE.json", "INPUT-ONLY-VECTORS.json", "REPORT.md", "SOURCE-MANIFESTS.json")
SOURCE_MANIFEST = "4048cff6b62a2756483ed59240a9a73395b6d5188fa53991c3541dc3da9d7947"
SOURCE_SEAL = "d1b300845d8492f41bb4c8afcb4715ef413d4bf523a90e9264dcc206959d4881"
TARGET_SHA256 = "3d66bcec36031fd48e6565d15f05fea656642377ca4f8c98cdce1cce8b7e95d2"
SOURCE_LEAF_SHA256 = "62dace058d92eaf8c4b9262c40d885d37801e55798d0be331e07bae0117a7081"


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
        "status": "SELECTED_PROCESS_BOUNDARY_STATIC; GENERIC_STORAGE_EVENT_JOIN_UNBOUND",
        "implementation_ready": False,
        "files": files,
    }
    raw = json.dumps(manifest, ensure_ascii=False, indent=2) + "\n"
    (ROOT / "MANIFEST.json").write_text(raw, encoding="utf-8", newline="\n")
    (ROOT / "MANIFEST.sha256").write_text(f"{sha256(ROOT / 'MANIFEST.json')}  MANIFEST.json\n", encoding="ascii", newline="\n")


def check() -> None:
    manifest = load_json("MANIFEST.json")
    require(manifest.get("package_id") == PACKAGE_ID, "package id mismatch")
    require(manifest.get("source_reverse_manifest_sha256") == SOURCE_MANIFEST, "source manifest mismatch")
    require(manifest.get("status") == "SELECTED_PROCESS_BOUNDARY_STATIC; GENERIC_STORAGE_EVENT_JOIN_UNBOUND", "status mismatch")
    require(manifest.get("implementation_ready") is False, "must not claim implementation readiness")
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
    require(evidence["source_package"]["evidence_manifest_sha256"] == SOURCE_MANIFEST, "evidence source mismatch")
    require(evidence["source_package"]["package_seal_sha256"] == SOURCE_SEAL, "source seal mismatch")
    require(evidence["target"]["sha256"] == TARGET_SHA256, "target image mismatch")
    require(evidence["approved_logical_rule"]["source_leaf_sha256"] == SOURCE_LEAF_SHA256, "source leaf mismatch")
    require(evidence["selected_native_process"]["selected_process_byte_layout"] == "NOT_APPLICABLE_TO_THIS_PROCESS_PATH", "layout status mismatch")
    require(evidence["separate_generic_storage_surface"]["selected_category5_producer_join"] == "UNBOUND", "generic event join must remain UNBOUND")
    require(evidence["execution_and_privacy"]["native_execution_performed"] is False, "must preserve no-run status")
    require(evidence["execution_and_privacy"]["implementation_ready"] is False, "must remain not ready")

    vectors = load_json("INPUT-ONLY-VECTORS.json")
    require(vectors["vectors"][0]["expected"]["drc_control_fifo_write"] is False, "approved decision vector mismatch")
    require(vectors["vectors"][1]["expected"]["selected_category5_record_write"] is False, "selected process vector mismatch")
    require(vectors["vectors"][2]["expected"]["selected_category5_producer_join"] == "UNBOUND", "generic surface vector mismatch")
    require(vectors["lineage"]["evidence_manifest_sha256"] == SOURCE_MANIFEST, "vector source mismatch")

    sources = load_json("SOURCE-MANIFESTS.json")
    require(sources["incoming_sealed_package"]["evidence_manifest_sha256"] == SOURCE_MANIFEST, "incoming manifest mismatch")
    require(sources["incoming_sealed_package"]["package_seal_sha256"] == SOURCE_SEAL, "incoming seal mismatch")
    require(sources["approved_source_leaf"]["sha256"] == SOURCE_LEAF_SHA256, "approved leaf hash mismatch")
    require(sources["copied_pcm_payload_or_key"] is False, "sensitive payload must not be copied")
    print(f"PASS package={PACKAGE_ID} selected_fifo_write=false generic_event_join=UNBOUND")


if __name__ == "__main__":
    if sys.argv[1:] == ["--build-manifest"]:
        build_manifest()
    elif not sys.argv[1:]:
        check()
    else:
        raise SystemExit("usage: python CHECK.py [--build-manifest]")
