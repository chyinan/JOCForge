from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "ADM-CONVERTER-CHANNEL-ID-TO-BWF-SLOT-CROSSWALK-V1"
PAYLOADS = ("EVIDENCE.json", "INPUT-ONLY-VECTORS.json", "REPORT.md", "SOURCE-MANIFESTS.json")
SOURCE_MANIFEST = "70eb0c02891fb3963d575f3783e809d3c7840b3ee0aa7e78cef0d7191bbc0c48"
CONVERTER_MANIFEST = "b98a0010672b79434fece33d8ca4576faa6e582096aeb44c2943759572118813"
INPUT_CROSSWALK = "4ef5495b80167803d1c45247b659d0644464b447894c10501c4e6d924c82d6c9"
CONVERTER_IMAGE = "9972be4f952ca5f9121fa83ef941b29d4c1e6baf0ff1f708baaaedffa1bd76a8"
INPUT_SHA256 = "7826dcbef37ef69642a6164c97a37bcfb3f917cea74aa0f0a30d76d0a825f1d1"


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
        "status": "STATIC_BWF_SLOT_CROSSWALK_BOUND; NATIVE_ROW_CONSUMPTION_UNBOUND",
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
    require(manifest.get("status") == "STATIC_BWF_SLOT_CROSSWALK_BOUND; NATIVE_ROW_CONSUMPTION_UNBOUND", "status mismatch")
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
    require(evidence["input_identity"]["sample_sha256_from_prior_receipt"] == INPUT_SHA256, "input identity mismatch")
    require(evidence["static_converter_mapping"]["disk_converter_image_sha256"] == CONVERTER_IMAGE, "converter image mismatch")
    require(evidence["static_converter_mapping"]["descriptor_slot_formula"] == "CHNA track_index - 1", "descriptor formula mismatch")
    require(evidence["input_crosswalk"]["bwf_interleaved_slot_formula"] == "CHNA track_index - 1", "BWF slot formula mismatch")
    require(evidence["input_crosswalk"]["numeric_equality_used_as_identity"] is False, "numeric equality must not imply identity")
    require(evidence["runtime_boundary"]["actual_reader_row_selection"] == "NOT_OBSERVED", "reader observation mismatch")
    require(evidence["runtime_boundary"]["meter_descriptor_join"] == "UNBOUND", "meter join mismatch")

    vectors = load_json("INPUT-ONLY-VECTORS.json")
    require(vectors["vectors"][0]["expected"]["bwf_interleaved_slot_0based"] == 9, "bed vector mismatch")
    require(vectors["vectors"][1]["expected"]["bwf_interleaved_slot_0based"] == 10, "object vector mismatch")
    require(vectors["limits"][1].find("not an identity rule") >= 0, "identity limit missing")
    require(vectors["lineage"]["source_manifest_sha256"] == SOURCE_MANIFEST, "vector source mismatch")

    sources = load_json("SOURCE-MANIFESTS.json")
    require(sources["incoming_package"]["manifest_sha256"] == SOURCE_MANIFEST, "source binding mismatch")
    require(sources["converter_mapping"]["manifest_sha256"] == CONVERTER_MANIFEST, "converter mapping mismatch")
    require(sources["input_metadata_crosswalk"]["manifest_sha256"] == INPUT_CROSSWALK, "input crosswalk mismatch")
    require(sources["copied_pcm_payload_or_audio_data"] is False, "PCM must not be copied")
    print(f"PASS package={PACKAGE_ID} rows=38 metadata_only=true native_read=UNBOUND")


if __name__ == "__main__":
    if sys.argv[1:] == ["--build-manifest"]:
        build_manifest()
    elif not sys.argv[1:]:
        check()
    else:
        raise SystemExit("usage: python CHECK.py [--build-manifest]")
