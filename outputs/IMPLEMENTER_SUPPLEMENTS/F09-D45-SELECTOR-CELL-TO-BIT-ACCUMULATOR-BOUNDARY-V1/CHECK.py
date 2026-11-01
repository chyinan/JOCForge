from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "F09-D45-SELECTOR-CELL-TO-BIT-ACCUMULATOR-BOUNDARY-V1"
PAYLOADS = ("EVIDENCE.json", "INPUT-ONLY-VECTORS.json", "REPORT.md", "SOURCE-MANIFESTS.json")
SOURCE_MANIFEST = "3a525c19200e50424d4109013f898ca59b34a11d52ecc11fb21c6885e56cdb4a"
TARGET_IMAGE = "1767070831dc0d5925a1fc401684ae1447cef2d6bc59659f8a25f53c95f482e9"


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
        "status": "SELECTOR_TO_BIT_ACCUMULATOR_STATIC_BOUND; SEMANTIC_FIELD_AND_OFFSET_UNBOUND",
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
    require(manifest.get("status") == "SELECTOR_TO_BIT_ACCUMULATOR_STATIC_BOUND; SEMANTIC_FIELD_AND_OFFSET_UNBOUND", "status mismatch")
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
    require(evidence["target_image"]["sha256"] == TARGET_IMAGE, "target image mismatch")
    require(evidence["selector_cell"]["lane0_block0"] == "owner+0x1bb22", "cell base mismatch")
    require(evidence["selector_cell"]["address_formula"] == "owner+0x1bb22 + 10*block + 2*lane", "cell geometry mismatch")
    require(evidence["bit_writer"]["function"] == "FUN_1802fd640", "writer mismatch")
    require(evidence["bit_writer"]["field_name"] == "UNBOUND", "field name must remain UNBOUND")
    require(evidence["bit_writer"]["absolute_stream_bit_range"] == "UNBOUND", "bit range must remain UNBOUND")
    require(evidence["execution_scope"]["implementation_ready"] is False, "must not claim readiness")

    vectors = load_json("INPUT-ONLY-VECTORS.json")
    require(vectors["vectors"][0]["expected"]["symbol_entry"] == "owner+0x140f0+0x2c8", "symbol table mismatch")
    require(vectors["vectors"][1]["expected"]["code_width_bits"] == 4, "code width mismatch")
    require(vectors["vectors"][1]["expected"]["symbol_width_bits"] == 7, "symbol width mismatch")
    require(vectors["vectors"][1]["expected"]["absolute_stream_bit_range"] == "UNBOUND", "vector must preserve unbound offset")
    require(vectors["lineage"]["source_manifest_sha256"] == SOURCE_MANIFEST, "vector source mismatch")

    sources = load_json("SOURCE-MANIFESTS.json")
    require(sources["incoming_package"]["manifest_sha256"] == SOURCE_MANIFEST, "source binding mismatch")
    require(sources["target_image"]["sha256"] == TARGET_IMAGE, "source image mismatch")
    require(sources["copied_raw_analysis_or_media"] is False, "raw source/media must not be copied")
    print(f"PASS package={PACKAGE_ID} selector_to_accumulator=BOUND field_and_absolute_range=UNBOUND")


if __name__ == "__main__":
    if sys.argv[1:] == ["--build-manifest"]:
        build_manifest()
    elif not sys.argv[1:]:
        check()
    else:
        raise SystemExit("usage: python CHECK.py [--build-manifest]")
