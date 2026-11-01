from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "F06-R138-PROCESS-STATE-FC8-PRODUCER-BOUNDARY-V1"
PAYLOADS = ("EVIDENCE.json", "INPUT-ONLY-VECTORS.json", "REPORT.md", "SOURCE-MANIFESTS.json")
SOURCE_MANIFEST = "c16759145be9eeb9337cf54373dbe096e1e7415bb2373e020c099a66d23d2a01"
SUPERSEDED_MANIFEST = "21d3b6b4e790e2820bce1ebd5283b24c319d467496dacb4cc3b93ad881f3424e"
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
        "status": "FC8_DWORD_WRITERS_BOUND; R138_DECODER_SOURCE_UNBOUND; PLUS_7E4_RETRACTED",
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
    require(manifest.get("status") == "FC8_DWORD_WRITERS_BOUND; R138_DECODER_SOURCE_UNBOUND; PLUS_7E4_RETRACTED", "status mismatch")
    require(manifest.get("implementation_ready") is False, "must remain not ready")
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
    require(evidence["supersedes"]["wash_manifest_sha256"] == SUPERSEDED_MANIFEST, "superseded package mismatch")
    require(evidence["target"]["image_sha256"] == TARGET_IMAGE, "target image mismatch")
    require(evidence["target"]["r138_job_xml_sha256"] == JOB_SHA256, "job hash mismatch")
    require(evidence["target"]["input_adm_sha256"] == ADM_SHA256, "input hash mismatch")
    require(evidence["offset_correction"]["parameter_type"] == "longlong*", "pointer type mismatch")
    require(evidence["offset_correction"]["byte_offset"] == "+0xfc8", "corrected byte offset mismatch")
    require(evidence["offset_correction"]["actual_memory_access_width"] == "32-bit DWORD", "access width mismatch")
    require(evidence["remaining_source_join"]["process_state_plus_0xfc8_to_r138_job_decoder"] == "UNBOUND", "job decoder join must remain UNBOUND")
    require(evidence["remaining_source_join"]["custom_bsi_zero_applicability"] == "UNBOUND", "zero applicability must remain UNBOUND")

    vectors = load_json("INPUT-ONLY-VECTORS.json")
    require(vectors["vectors"][0]["expected"]["byte_offset"] == "+0xfc8", "vector offset mismatch")
    require(vectors["vectors"][1]["expected"]["constructor_zero_proves_final_value"] is False, "constructor default must not imply final value")
    require(vectors["lineage"]["source_manifest_sha256"] == SOURCE_MANIFEST, "vector source mismatch")
    sources = load_json("SOURCE-MANIFESTS.json")
    require(sources["incoming_package"]["manifest_sha256"] == SOURCE_MANIFEST, "source binding mismatch")
    require(sources["superseded_wash_supplement"]["manifest_sha256"] == SUPERSEDED_MANIFEST, "superseded binding mismatch")
    require(sources["raw_trace_or_media_copied"] is False, "raw material must not be copied")
    print(f"PASS package={PACKAGE_ID} corrected_offset=+0xfc8 R138_origin=UNBOUND")


if __name__ == "__main__":
    if sys.argv[1:] == ["--build-manifest"]:
        build_manifest()
    elif not sys.argv[1:]:
        check()
    else:
        raise SystemExit("usage: python CHECK.py [--build-manifest]")
