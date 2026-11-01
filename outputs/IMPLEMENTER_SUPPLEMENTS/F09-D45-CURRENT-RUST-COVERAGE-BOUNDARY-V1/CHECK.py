from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "F09-D45-CURRENT-RUST-COVERAGE-BOUNDARY-V1"
PAYLOADS = ("EVIDENCE.json", "INPUT-ONLY-VECTORS.json", "REPORT.md", "SOURCE-MANIFESTS.json")
SOURCE_MANIFEST = "74b07756b07c1c2c73268d095b8fd32d845856bafce5d67e18e0a2064f996644"
F09_MANIFEST = "a5df1c8de351fa6ca59b1d6600805fd71cc79d83c18fc07663a89f62de669164"
F12_MANIFEST = "01dbb8fe97bc8f40247340b93eccd008826c2330e6c4616323f2cbd1aa2ab81f"
QUANTIZER_MANIFEST = "fd819c3ec27e7bade55efda5e648792c9c33892a9be7a36bbfce89231b9e961f"


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
        "status": "F12_SOURCE_PATH_IMPLEMENTED; D45_PATH_MISSING_IN_SCANNED_SOURCE; MISMATCH_CAUSE_UNBOUND",
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
    require(manifest.get("status") == "F12_SOURCE_PATH_IMPLEMENTED; D45_PATH_MISSING_IN_SCANNED_SOURCE; MISMATCH_CAUSE_UNBOUND", "status mismatch")
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
    require(evidence["approved_contracts"]["f09_d45_manifest_sha256"] == F09_MANIFEST, "F09 contract mismatch")
    require(evidence["approved_contracts"]["f12_manifest_sha256"] == F12_MANIFEST, "F12 contract mismatch")
    require(evidence["approved_contracts"]["f12_quantizer_contract_sha256"] == QUANTIZER_MANIFEST, "quantizer contract mismatch")
    require(evidence["current_source"]["f12_quantization"] == "IMPLEMENTED", "F12 coverage mismatch")
    require(evidence["current_source"]["f09_d45_u16_selector_to_symbol_to_4_7_bit_path"] == "MISSING_IN_SCANNED_CURRENT_SOURCE", "D45 status mismatch")
    require(evidence["current_source"]["dB_per_bit_selector"] == "SEPARATE_IMPLEMENTED_PATH; not identified with D45", "selector distinction mismatch")
    require(evidence["attempt10"]["hash_mismatch_causality"] == "NOT_ESTABLISHED", "causality must remain unestablished")

    vectors = load_json("INPUT-ONLY-VECTORS.json")
    require(vectors["vectors"][0]["expected"]["current_rust_status"] == "IMPLEMENTED", "F12 vector mismatch")
    require(vectors["vectors"][1]["expected"]["current_rust_status"] == "MISSING_IN_SCANNED_CURRENT_SOURCE", "D45 vector mismatch")
    require(vectors["vectors"][2]["expected"]["identity_with_d45_selector_proven"] is False, "selector distinction vector mismatch")
    require(vectors["lineage"]["source_manifest_sha256"] == SOURCE_MANIFEST, "vector source mismatch")
    sources = load_json("SOURCE-MANIFESTS.json")
    require(sources["incoming_package"]["manifest_sha256"] == SOURCE_MANIFEST, "source binding mismatch")
    require(sources["contracts"][0]["manifest_sha256"] == F09_MANIFEST, "F09 source mismatch")
    require(sources["contracts"][1]["manifest_sha256"] == F12_MANIFEST, "F12 source mismatch")
    require(sources["raw_prototype_source_or_output_copied"] is False, "raw sources must not be copied")
    print(f"PASS package={PACKAGE_ID} F12=IMPLEMENTED D45=UNFOUND_IN_SCAN mismatch_cause=UNBOUND")


if __name__ == "__main__":
    if sys.argv[1:] == ["--build-manifest"]:
        build_manifest()
    elif not sys.argv[1:]:
        check()
    else:
        raise SystemExit("usage: python CHECK.py [--build-manifest]")
