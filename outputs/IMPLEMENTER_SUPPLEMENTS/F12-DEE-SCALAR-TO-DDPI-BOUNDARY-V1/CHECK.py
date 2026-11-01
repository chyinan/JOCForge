from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "F12-DEE-SCALAR-TO-DDPI-BOUNDARY-V1"
PAYLOADS = ("EVIDENCE.json", "INPUT-ONLY-VECTORS.json", "REPORT.md", "SOURCE-MANIFESTS.json")
SOURCE_MANIFEST = "d7deb53845cf3c484e222e069a819c33f82d383510fc0529ee8efc412505150d"
MODEL_MANIFEST = "0afba77679f20496cae42b840252c7278fad5185274fa0fd81e1e95b9f2401a2"
MODEL_CONTRACT = "30e3a5e3d1f9e22cd6c8658bc968f6217bd83c4072260192b7aa7328c62cde61"
DDPI_SUPPLEMENT = "da53268b8f92c4b84a0684a7d4f9e7e73c3fce268c4bd4e2c1519326a9c7a5c4"
TARGET_IMAGE = "3d66bcec36031fd48e6565d15f05fea656642377ca4f8c98cdce1cce8b7e95d2"


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
        "status": "STATIC_NORMALIZER_BOUND; ID12_AND_BSI_JOIN_UNBOUND",
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
    require(manifest.get("status") == "STATIC_NORMALIZER_BOUND; ID12_AND_BSI_JOIN_UNBOUND", "status mismatch")
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
    require(evidence["approved_model"]["manifest_sha256"] == MODEL_MANIFEST, "model manifest mismatch")
    require(evidence["approved_model"]["contract_sha256"] == MODEL_CONTRACT, "model contract mismatch")
    require(evidence["native_target"]["sha256"] == TARGET_IMAGE, "target image mismatch")
    require(evidence["ddpi_and_wire_boundary"]["native_scalar_to_id_0x12"] == "UNBOUND", "ID12 join must remain UNBOUND")
    require(evidence["ddpi_and_wire_boundary"]["id_0x12_to_f12_bsi_u5_bits_45_49"] == "UNBOUND", "BSI join must remain UNBOUND")
    require(evidence["existing_sample_comparisons"]["proves_native_formula"] is False, "sample matches must remain conditional")
    require(evidence["execution_scope"]["implementation_ready"] is False, "must not claim readiness")

    vectors = load_json("INPUT-ONLY-VECTORS.json")
    require(vectors["vectors"][0]["expected"]["helper_signed_result"] == -21, "ordinary vector mismatch")
    require(vectors["vectors"][1]["expected"]["helper_signed_result"] == -120, "sentinel vector mismatch")
    require(vectors["vectors"][1]["expected"]["model_u5_31_implemented"] is False, "sentinel distinction mismatch")
    require(vectors["lineage"]["source_manifest_sha256"] == SOURCE_MANIFEST, "vector source mismatch")

    sources = load_json("SOURCE-MANIFESTS.json")
    require(sources["incoming_package"]["manifest_sha256"] == SOURCE_MANIFEST, "source binding mismatch")
    require(sources["approved_model_source"]["manifest_sha256"] == MODEL_MANIFEST, "model source binding mismatch")
    require(sources["prior_wash_ddpi_boundary"]["manifest_sha256"] == DDPI_SUPPLEMENT, "DDPi supplement mismatch")
    require(sources["raw_ghidra_trace_or_script_copied"] is False, "raw trace/script must not be copied")
    print(f"PASS package={PACKAGE_ID} native_normalizer=BOUND ID12_BSI=UNBOUND")


if __name__ == "__main__":
    if sys.argv[1:] == ["--build-manifest"]:
        build_manifest()
    elif not sys.argv[1:]:
        check()
    else:
        raise SystemExit("usage: python CHECK.py [--build-manifest]")
