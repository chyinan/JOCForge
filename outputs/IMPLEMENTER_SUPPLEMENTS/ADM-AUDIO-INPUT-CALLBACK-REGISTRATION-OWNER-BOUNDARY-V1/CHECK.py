from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "ADM-AUDIO-INPUT-CALLBACK-REGISTRATION-OWNER-BOUNDARY-V1"
PAYLOADS = ("EVIDENCE.json", "INPUT-ONLY-VECTORS.json", "REPORT.md", "SOURCE-MANIFESTS.json")
SOURCE_MANIFEST = "146e871a8a6ffa9abfd7b41910a6cedf2ddd646dbb2110923a1a759fccc39142"
UPSTREAM_MANIFEST = "abadcde4fbae11a5babf18cf136873d5673b8d7ad9f9c8da3ff843f5fc70e1cd"
DEE_SHA256 = "65485b6d35fa534b25109ab6f238971aaded73fc99c7f7fb119d1a5d5299234e"
DDP_SHA256 = "2fd503793f68dfa13037adf4082b8e5327e5046abfb6183ee84d7ae5839ecd72"


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
        "status": "STATIC_GETTER_REGISTRY_BOUND; SELECTED_CALLBACK_OWNER_AND_READER_JOIN_UNBOUND",
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
    require(manifest.get("status") == "STATIC_GETTER_REGISTRY_BOUND; SELECTED_CALLBACK_OWNER_AND_READER_JOIN_UNBOUND", "status mismatch")
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
    require(evidence["target_host"]["sha256"] == DEE_SHA256, "host hash mismatch")
    require(evidence["getter"]["return_record"]["entry_stride_bytes"] == 8, "getter stride mismatch")
    require(evidence["registration"]["shared_vector_offset"] == "+0x18", "registry offset mismatch")
    require(evidence["registration"]["typed_input_owner_retained"] is False, "typed owner must remain absent")
    require(evidence["open_joins"]["selected_callback_to_atmosmezz_reader"] == "UNBOUND", "reader join mismatch")
    require(evidence["open_joins"]["same_task_generation"] == "UNBOUND", "generation join mismatch")
    require(evidence["execution_scope"]["implementation_ready"] is False, "must remain not ready")

    vectors = load_json("INPUT-ONLY-VECTORS.json")
    require(vectors["vectors"][0]["expected"]["entry_stride_bytes"] == 8, "vector stride mismatch")
    require(vectors["vectors"][2]["expected"]["domains_proven_same"] is False, "ordinal domains must remain separate")
    require(vectors["lineage"]["source_manifest_sha256"] == SOURCE_MANIFEST, "vector lineage mismatch")
    sources = load_json("SOURCE-MANIFESTS.json")
    require(sources["incoming_package"]["manifest_sha256"] == SOURCE_MANIFEST, "source binding mismatch")
    require(sources["upstream_reader_map"]["manifest_sha256"] == UPSTREAM_MANIFEST, "upstream mapping mismatch")
    require(sources["target_images"][1]["sha256"] == DDP_SHA256, "DDP image mismatch")
    require(sources["copied_raw_analysis_or_media"] is False, "raw material must not be copied")
    print(f"PASS package={PACKAGE_ID} getter_registry=BOUND typed_reader_join=UNBOUND")


if __name__ == "__main__":
    if sys.argv[1:] == ["--build-manifest"]:
        build_manifest()
    elif not sys.argv[1:]:
        check()
    else:
        raise SystemExit("usage: python CHECK.py [--build-manifest]")
