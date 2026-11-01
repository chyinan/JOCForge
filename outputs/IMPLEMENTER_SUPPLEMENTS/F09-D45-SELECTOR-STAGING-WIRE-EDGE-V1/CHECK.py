from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "F09-D45-SELECTOR-STAGING-WIRE-EDGE-V1"
PAYLOADS = (
    "EVIDENCE.json",
    "INPUT-ONLY-VECTORS.json",
    "REPORT.md",
    "SOURCE-MANIFESTS.json",
)
SOURCE_MANIFEST = "d7942feb8eade85905562a76c5e4afe201dbb7a162094a21b69a02fc5faa6f42"
PRIOR_MANIFEST = "6c3218735a9da7964cac061611779c61cf997ad011bef6b840a97497922e8933"
TARGET_IMAGE_SHA256 = "1767070831dc0d5925a1fc401684ae1447cef2d6bc59659f8a25f53c95f482e9"


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
        "status": "STATIC_STAGING_CHAIN_BOUND; WIRE_FIELD_UNBOUND",
        "implementation_ready": False,
        "files": files,
    }
    raw = json.dumps(manifest, ensure_ascii=False, indent=2) + "\n"
    (ROOT / "MANIFEST.json").write_text(raw, encoding="utf-8", newline="\n")
    seal = sha256(ROOT / "MANIFEST.json")
    (ROOT / "MANIFEST.sha256").write_text(f"{seal}  MANIFEST.json\n", encoding="ascii", newline="\n")


def check() -> None:
    manifest = load_json("MANIFEST.json")
    require(manifest.get("package_id") == PACKAGE_ID, "package id mismatch")
    require(manifest.get("source_reverse_manifest_sha256") == SOURCE_MANIFEST, "source manifest mismatch")
    require(manifest.get("status") == "STATIC_STAGING_CHAIN_BOUND; WIRE_FIELD_UNBOUND", "status mismatch")
    require(manifest.get("implementation_ready") is False, "must remain not implementation-ready")
    entries = manifest.get("files")
    require(isinstance(entries, list), "manifest files must be a list")
    require({entry.get("file") for entry in entries} == set(PAYLOADS), "payload set mismatch")
    for entry in entries:
        path = ROOT / entry["file"]
        require(path.is_file(), f"missing payload: {entry['file']}")
        require(path.stat().st_size == entry.get("bytes"), f"byte count mismatch: {entry['file']}")
        require(sha256(path) == entry.get("sha256"), f"payload hash mismatch: {entry['file']}")

    seal_line = (ROOT / "MANIFEST.sha256").read_text(encoding="ascii").strip()
    require(seal_line == f"{sha256(ROOT / 'MANIFEST.json')}  MANIFEST.json", "manifest seal mismatch")

    evidence = load_json("EVIDENCE.json")
    require(evidence["source"]["wash_manifest_sha256"] == SOURCE_MANIFEST, "evidence source mismatch")
    require(evidence["source"]["prior_supplement_manifest_sha256"] == PRIOR_MANIFEST, "prior supplement mismatch")
    require(evidence["target"]["sha256"] == TARGET_IMAGE_SHA256, "target image mismatch")
    require(evidence["static_facts"]["lane0_block0_cell"] == "U16 at staging_base+0x3a; code domain 0..3", "cell contract mismatch")
    require(evidence["static_facts"]["last_recovered_read"] == "short(param_2+0x12)", "last read mismatch")
    require(evidence["static_facts"]["cell_minus_read_offset"] == "0x28", "offset distance mismatch")
    require(evidence["conclusion"]["ddp_ec3_field"] == "UNBOUND", "wire field must remain UNBOUND")
    require(evidence["conclusion"]["ddp_ec3_bit_range"] == "UNBOUND", "wire bit range must remain UNBOUND")
    require(evidence["conclusion"]["implementation_ready"] is False, "evidence cannot be implementation-ready")

    vectors = load_json("INPUT-ONLY-VECTORS.json")
    require(len(vectors.get("vectors", [])) == 2, "expected two structural vectors")
    first = vectors["vectors"][0]
    require(first["expected"]["cell_offset_from_base"] == "0x3a", "vector cell offset mismatch")
    require(first["expected"]["last_read_offset_from_base"] == "0x12", "vector read offset mismatch")
    require(first["expected"]["byte_distance"] == "0x28", "vector delta mismatch")
    require(first["expected"]["wire_field"] == "UNBOUND", "vector must not invent a wire field")
    second = vectors["vectors"][1]
    require(second["expected"]["selector_cell_serialization_proven"] is False, "serializer edge must remain unproven")
    require(vectors["lineage"]["prior_manifest_sha256"] == PRIOR_MANIFEST, "vector lineage mismatch")

    source = load_json("SOURCE-MANIFESTS.json")
    require(source["incoming_wash_package"]["manifest_sha256"] == SOURCE_MANIFEST, "source binding mismatch")
    require(source["prior_implementer_supplement"]["manifest_sha256"] == PRIOR_MANIFEST, "prior binding mismatch")
    require(source["target_image"]["sha256"] == TARGET_IMAGE_SHA256, "source target mismatch")
    require(source["copied_source_or_media"] is False, "raw source/media must not be copied")
    print(f"PASS package={PACKAGE_ID} payloads={len(PAYLOADS)} source={SOURCE_MANIFEST} wire=UNBOUND")


if __name__ == "__main__":
    if sys.argv[1:] == ["--build-manifest"]:
        build_manifest()
    elif not sys.argv[1:]:
        check()
    else:
        raise SystemExit("usage: python CHECK.py [--build-manifest]")
