from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "F13C-CALLBACK-INPUT-OUTPUT-PAIR-IDENTITY-BOUNDARY-V1"
PAYLOADS = ("EVIDENCE.json", "INPUT-ONLY-VECTORS.json", "REPORT.md", "SOURCE-MANIFESTS.json")
SOURCE_MANIFEST = "aa6ec1ec9a96d823fb96025599829374912b0ad885c63cf64dfe6c2c12a162c3"
PARENT_MANIFEST = "7cee591c7649aaa2d285ec0cd88697d01ca31cd2b85832d2a13c4e23df29c45f"
IMAGE_HASH = "4b4c624d94a5b47d97b62b28073e1bd351b2deeeea039a7ccfaa2af96562c8a7"


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
        "status": "INPUT_OUTPUT_PAIR_DISTINCT; OUTPUT_CONSUMER_AND_WRITER_UNBOUND",
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
    require(manifest.get("status") == "INPUT_OUTPUT_PAIR_DISTINCT; OUTPUT_CONSUMER_AND_WRITER_UNBOUND", "status mismatch")
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
    require(evidence["source"]["parent_v13_manifest_sha256"] == PARENT_MANIFEST, "parent mismatch")
    require(evidence["source"]["project_recorded_image_sha256"] == IMAGE_HASH, "image identity mismatch")
    require(evidence["source"]["installed_image_rehashed"] is False, "do not upgrade image claim")
    require(evidence["input_pair_path"]["candidate_slot_callsite"] == "0x1403c1bb9", "candidate callsite mismatch")
    require(evidence["input_pair_path"]["receiver"] == "object from input param_3 pair", "receiver mismatch")
    require(evidence["output_pair_path"]["identity_with_input_param3_pair"] == "DISTINCT_LOCAL_PATH; NO SHARED POINTER JOIN", "pair distinction mismatch")
    require(evidence["open_edges"]["first_consumer_of_output_fields"] == "UNBOUND", "consumer must remain UNBOUND")
    require(evidence["open_edges"]["final_f13c_writer"] == "UNBOUND", "writer must remain UNBOUND")
    require(evidence["execution_scope"]["implementation_ready"] is False, "must remain not ready")

    vectors = load_json("INPUT-ONLY-VECTORS.json")
    require(vectors["vectors"][0]["expected"]["slot_0xf0_call_receiver"] == "input pair object", "input pair vector mismatch")
    require(vectors["vectors"][1]["expected"]["first_consumer_of_callback_fields"] == "UNBOUND", "output consumer vector mismatch")
    require(vectors["lineage"]["source_manifest_sha256"] == SOURCE_MANIFEST, "vector source mismatch")
    sources = load_json("SOURCE-MANIFESTS.json")
    require(sources["incoming_package"]["manifest_sha256"] == SOURCE_MANIFEST, "source binding mismatch")
    require(sources["target_image"]["project_recorded_sha256"] == IMAGE_HASH, "source image mismatch")
    require(sources["copied_raw_analysis_or_media"] is False, "raw material must not be copied")
    print(f"PASS package={PACKAGE_ID} input_output_pair=distinct output_consumer=UNBOUND writer=UNBOUND")


if __name__ == "__main__":
    if sys.argv[1:] == ["--build-manifest"]:
        build_manifest()
    elif not sys.argv[1:]:
        check()
    else:
        raise SystemExit("usage: python CHECK.py [--build-manifest]")
