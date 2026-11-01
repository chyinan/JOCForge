from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "F13C-CALLBACK-CONTROL-BLOCK-SLOT-BOUNDARY-V1"
PAYLOADS = ("EVIDENCE.json", "INPUT-ONLY-VECTORS.json", "REPORT.md", "SOURCE-MANIFESTS.json")
SOURCE_MANIFEST = "f21807af691ba950aa88c0964fe6091b5ed3daff44be4362c4aefe119fc0dcbe"
PARENT_MANIFEST = "9b57083d1b97f38801e0906e88ec9b7ff336c7b1462d0581fd092f949e469d9a"
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
        "status": "STATIC_NEW_SLOT_BOUND;_OLD_RECEIVER_AND_WRITER_UNBOUND",
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
    require(manifest.get("status") == "STATIC_NEW_SLOT_BOUND;_OLD_RECEIVER_AND_WRITER_UNBOUND", "status mismatch")
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
    require(evidence["source"]["parent_v11_manifest_sha256"] == PARENT_MANIFEST, "parent mismatch")
    require(evidence["source"]["ghidra_project_image_sha256"] == IMAGE_HASH, "image identity mismatch")
    require(evidence["source"]["current_executable_rehashed"] is False, "do not upgrade image identity claim")
    require(evidence["typed_static_edge"]["slot_rva"] == "0x3f2a40", "slot target mismatch")
    require(evidence["receiver_join"]["old_receiver_table_identity"] == "UNBOUND", "old receiver must remain UNBOUND")
    require(evidence["downstream"]["preferred_downmix_relation"] == "UNBOUND", "downmix join must remain UNBOUND")
    require(evidence["downstream"]["f13c_writer"] == "UNBOUND", "F13C writer must remain UNBOUND")
    require(evidence["restrictions"]["disassembly_copied"] is False, "raw analysis must not be copied")

    vector_file = load_json("INPUT-ONLY-VECTORS.json")
    first = vector_file["vectors"][0]
    require(first["expected"]["new_control_block_slot_target_rva"] == "0x3f2a40", "vector slot mismatch")
    require(first["expected"]["old_receiver_same_table_proven"] is False, "vector must preserve old-receiver boundary")
    require(vector_file["lineage"]["source_manifest_sha256"] == SOURCE_MANIFEST, "vector lineage mismatch")

    sources = load_json("SOURCE-MANIFESTS.json")
    require(sources["incoming_package"]["manifest_sha256"] == SOURCE_MANIFEST, "source binding mismatch")
    require(sources["target_image"]["project_recorded_sha256"] == IMAGE_HASH, "source image mismatch")
    require(sources["copied_raw_analysis_or_media"] is False, "raw material must not be copied")
    print(f"PASS package={PACKAGE_ID} slot_rva=0x3f2a40 old_receiver=UNBOUND writer=UNBOUND")


if __name__ == "__main__":
    if sys.argv[1:] == ["--build-manifest"]:
        build_manifest()
    elif not sys.argv[1:]:
        check()
    else:
        raise SystemExit("usage: python CHECK.py [--build-manifest]")
