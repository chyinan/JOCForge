from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "ATMOS-JOC-ID0-RENDERER-VECTOR-OFFSET-ERRATUM-V1"
STATUS = "ID0_LOOKUP_VECTOR_OFFSET_RECONCILED; TWO_VECTORS_ONE_RENDERER"
SOURCE_MANIFESTS = {
    "atmos_joc_id0_renderer_constructor_edge_wash_v1": "c43917dcfcfc797eade20671602c9a7b246620fe2f3754bc5e6c0c16371ffb9e",
    "atmos_joc_selector_id0_config_wash_v1": "bcd366ee63916707421278c34ba430b8b35bede053e65f6f8de13963b35ce8bc",
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
    require(set(sources) == set(SOURCE_MANIFESTS), "source package inventory")
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
    require(evidence["renderer_layout"]["outer_record_vector"]["header_offsets"] == ["+0x60", "+0x68", "+0x70"], "outer vector")
    require(evidence["renderer_layout"]["nested_object_vector"]["destination_header"] == ["+0x78", "+0x80", "+0x88"], "nested vector")
    require(evidence["id0_lookup"]["begin_read"] == "receiver+0x18 = renderer+0x78", "ID0 begin offset")
    require(evidence["id0_lookup"]["end_read"] == "receiver+0x20 = renderer+0x80", "ID0 end offset")
    require(evidence["id0_lookup"]["capacity_read"].endswith("is not read by this scan"), "capacity boundary")
    require(evidence["correction"]["selected_id0_concrete_type"] == "UNBOUND/NO", "class boundary")
    require(evidence["correction"]["selected_id0_plus_0x58_writer"] == "UNBOUND/NO", "writer boundary")
    print(f"PASS package={PACKAGE_ID}")
    print(f"PASS manifest_sha256={manifest_hash}")
    print("PASS same renderer has distinct outer +0x60 and nested ID-0 vector +0x78")
    print("PASS selected ID-0 type and +0x58 writer remain UNBOUND")
    return 0


if __name__ == "__main__":
    main()
