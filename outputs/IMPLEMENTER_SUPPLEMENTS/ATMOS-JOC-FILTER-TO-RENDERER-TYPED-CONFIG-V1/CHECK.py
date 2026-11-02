from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "ATMOS-JOC-FILTER-TO-RENDERER-TYPED-CONFIG-V1"
STATUS = "FILTER_TO_RENDERER_TYPED_CONFIG_VECTOR_BOUND; ID0_ELEMENT_OWNER_AND_SELECTOR_WRITER_UNBOUND"
SOURCE_MANIFESTS = {
    "atmos_joc_id0_renderer_constructor_edge_wash_v1": "c43917dcfcfc797eade20671602c9a7b246620fe2f3754bc5e6c0c16371ffb9e",
    "atmos_joc_id0_renderer_vector_offset_erratum_wash_v1": "d1e7c940427d0eef48309c07a5c8eac882a997125c38f0428e4ed91b8bf32c21",
}


def require(condition: bool, label: str) -> None:
    if not condition:
        raise SystemExit(f"FAIL {label}")


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def load(name: str) -> dict:
    return json.loads((ROOT / name).read_text(encoding="utf-8"))


def main() -> int:
    manifest = load("MANIFEST.json")
    manifest_hash = sha256(ROOT / "MANIFEST.json")
    require(manifest_hash == (ROOT / "MANIFEST.sha256").read_text(encoding="ascii").split()[0], "manifest hash")
    require(manifest["package_id"] == PACKAGE_ID, "package id")
    require(manifest["status"] == STATUS, "status")
    sources = {row["package_id"]: row for row in manifest["source_packages"]}
    require(set(sources) == set(SOURCE_MANIFESTS), "source packages")
    for package_id, expected_hash in SOURCE_MANIFESTS.items():
        require(sources[package_id]["manifest_sha256"] == expected_hash, f"source manifest {package_id}")
    payload = {entry["file"]: entry for entry in manifest["files"]}
    actual = {path.name for path in ROOT.iterdir() if path.is_file() and path.name not in {"MANIFEST.json", "MANIFEST.sha256", "package_seal.json"}}
    require(actual == set(payload), "payload inventory")
    for name, entry in payload.items():
        path = ROOT / name
        require(path.stat().st_size == entry["bytes"], f"size {name}")
        require(sha256(path) == entry["sha256"], f"hash {name}")
    seal = load("package_seal.json")
    sealed = {entry["file"]: entry for entry in seal["files"]}
    require(seal["package_id"] == PACKAGE_ID and seal["file_count"] == len(sealed), "seal identity")
    require(set(sealed) == set(payload) | {"MANIFEST.json", "MANIFEST.sha256"}, "seal inventory")
    for name, entry in sealed.items():
        path = ROOT / name
        require(path.stat().st_size == entry["bytes"], f"sealed size {name}")
        require(sha256(path) == entry["sha256"], f"sealed hash {name}")

    evidence = load("EVIDENCE.json")
    require(evidence["status"] == STATUS, "evidence status")
    require(evidence["typed_renderer_vector"]["transfer_bound"] is True, "typed vector transfer")
    require(evidence["id0_lookup"]["scan_reads_begin"] == "receiver+0x18 = renderer+0x78", "nested lookup begin")
    require(evidence["id0_lookup"]["scan_reads_end"] == "receiver+0x20 = renderer+0x80", "nested lookup end")
    require(evidence["selected_id0_disposition"]["concrete_id0_element_constructor"] == "UNBOUND/NO", "ID0 element boundary")
    require(evidence["selected_id0_disposition"]["id0_plus_0x58_writer"] == "UNBOUND/NO", "selector writer boundary")
    require(evidence["execution"]["dee_run"] == "NOT_RUN", "run status")
    print(f"PASS package={PACKAGE_ID}")
    print(f"PASS manifest_sha256={manifest_hash}")
    print("PASS typed Filter->AtmosProcessor->renderer config vector transfer")
    print("PASS concrete ID-0 element/writer and run selector remain UNBOUND")
    return 0


if __name__ == "__main__":
    main()
