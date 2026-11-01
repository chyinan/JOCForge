from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "PCM24-DDP-SOURCE-PROPERTY-TYPED-MAP-LINEAGE-V1"
PAYLOADS = ("EVIDENCE.json", "INPUT-ONLY-VECTORS.json", "REPORT.md", "SOURCE-MANIFESTS.json")
SOURCE_MANIFEST = "1ab196b4b28c634da1465ce634d30c34766e5cb240f39111ed06f0900fc9f65a"
PRIOR_MANIFEST = "40f4d205760745942b2f3e4b1959a6c4908c5d311428c866d979f7418bac5e9d"
CAPTURE_MANIFEST = "d30fa0500bc0e1fbd1351ecec125af7c0cf721f1e43fc71319b78ba91c33da14"
CURRENT_IMAGE = "2fd503793f68dfa13037adf4082b8e5327e5046abfb6183ee84d7ae5839ecd72"
PRIOR_IMAGE = "3d66bcec36031fd48e6565d15f05fea656642377ca4f8c98cdce1cce8b7e95d2"


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
        "status": "STATIC_TYPED_PROPERTY_PATH_BOUND; HOST_WRITER_JOIN_AND_RUNTIME_UNBOUND",
        "implementation_ready": False,
        "files": files,
    }
    raw = json.dumps(manifest, ensure_ascii=False, indent=2) + "\n"
    (ROOT / "MANIFEST.json").write_text(raw, encoding="utf-8", newline="\n")
    (ROOT / "MANIFEST.sha256").write_text(f"{sha256(ROOT / 'MANIFEST.json')}  MANIFEST.json\n", encoding="ascii", newline="\n")


def check() -> None:
    manifest = load_json("MANIFEST.json")
    require(manifest.get("package_id") == PACKAGE_ID, "package id mismatch")
    require(manifest.get("source_reverse_manifest_sha256") == SOURCE_MANIFEST, "source package mismatch")
    require(manifest.get("status") == "STATIC_TYPED_PROPERTY_PATH_BOUND; HOST_WRITER_JOIN_AND_RUNTIME_UNBOUND", "status mismatch")
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
    require(evidence["source"]["manifest_sha256"] == SOURCE_MANIFEST, "evidence source mismatch")
    require(evidence["source"]["prior_wash_supplement_manifest_sha256"] == PRIOR_MANIFEST, "prior wash mismatch")
    require(evidence["source"]["linked_capture_manifest_sha256"] == CAPTURE_MANIFEST, "capture binding mismatch")
    require(evidence["target_image"]["sha256"] == CURRENT_IMAGE, "current image mismatch")
    require(evidence["source_property"]["parser_lookup_map_offset"] == "state+0x1530", "parser map field mismatch")
    require(evidence["source_property"]["normalized_string_destination"] == "outer+0xc0", "string destination mismatch")
    require(evidence["media_metadata"]["stored_type"] == "SafeType<vector<map<string,string>>>", "metadata type mismatch")
    require(evidence["open_joins"]["metadata_map_to_host_source_writer_param3"] == "UNBOUND", "writer join must remain UNBOUND")
    require(evidence["open_joins"]["runtime_setter_value"] == "NOT_CAPTURED", "runtime setter must remain NOT_CAPTURED")
    require(evidence["restrictions"]["decompilation_copied"] is False, "decompilation must not be copied")
    require(evidence["restrictions"]["implementation_ready"] is False, "evidence must not claim readiness")

    vector_file = load_json("INPUT-ONLY-VECTORS.json")
    vector = vector_file["vectors"][0]
    require(vector["input"]["property_key"] == "dee_audio_filter_source", "vector property mismatch")
    require(vector["expected"]["normalized_string_directly_passed_as_metadata_map"] is False, "vector conflates branches")
    require(vector["expected"]["map_to_prior_host_writer_param3"] == "UNBOUND", "vector writer join mismatch")
    require(vector_file["lineage"]["current_image_sha256"] == CURRENT_IMAGE, "vector current image mismatch")
    require(vector_file["lineage"]["previous_writer_image_sha256"] == PRIOR_IMAGE, "vector prior image mismatch")

    sources = load_json("SOURCE-MANIFESTS.json")
    require(sources["incoming_wash_package"]["manifest_sha256"] == SOURCE_MANIFEST, "source binding mismatch")
    require(sources["previous_wash_supplement"]["manifest_sha256"] == PRIOR_MANIFEST, "prior binding mismatch")
    require(sources["linked_capture"]["manifest_sha256"] == CAPTURE_MANIFEST, "linked capture mismatch")
    require(sources["raw_decompilation_or_media_copied"] is False, "raw material must not be copied")
    print(f"PASS package={PACKAGE_ID} current_image={CURRENT_IMAGE} writer_join=UNBOUND runtime=NOT_CAPTURED")


if __name__ == "__main__":
    if sys.argv[1:] == ["--build-manifest"]:
        build_manifest()
    elif not sys.argv[1:]:
        check()
    else:
        raise SystemExit("usage: python CHECK.py [--build-manifest]")
