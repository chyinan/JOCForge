from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "F12-OPEN-CONFIG-GUARD-AND-DDPI-ID12-BSI-BOUNDARY-V1"
PAYLOADS = ("EVIDENCE.json", "INPUT-ONLY-VECTORS.json", "REPORT.md", "SOURCE-MANIFESTS.json")
SOURCE_MANIFEST = "4b0cbdb03e8bb5fa73e330dc60d13dc7bbb0ac754c3325fb638b63cdb549754d"
MODEL_MANIFEST = "0afba77679f20496cae42b840252c7278fad5185274fa0fd81e1e95b9f2401a2"
DDPI_SUPPLEMENT = "da53268b8f92c4b84a0684a7d4f9e7e73c3fce268c4bd4e2c1519326a9c7a5c4"
EMPIRE_IMAGE = "3d66bcec36031fd48e6565d15f05fea656642377ca4f8c98cdce1cce8b7e95d2"
DDPI_IMAGE = "0cf23520799f52323b0994e5fcc16924da7b027a5b83a0cff3b4f06792c18532"


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
        "status": "OPEN_GUARD_ZERO_INIT_STATIC; ID12_TO_BSI_JOIN_UNBOUND",
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
    require(manifest.get("status") == "OPEN_GUARD_ZERO_INIT_STATIC; ID12_TO_BSI_JOIN_UNBOUND", "status mismatch")
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
    require(evidence["targets"][0]["sha256"] == EMPIRE_IMAGE, "Empire image mismatch")
    require(evidence["targets"][1]["sha256"] == DDPI_IMAGE, "DDPi image mismatch")
    require(evidence["open_config_guard"]["resolved_closure_accesses"] == "one byte read; no write", "open guard scope mismatch")
    require(evidence["ddpi_id12"]["field"] == "32-bit per-stream state+0x9c", "ID12 field mismatch")
    require(evidence["serializer_boundary"]["id12_to_bsi_u5_consumer"] == "UNBOUND", "BSI join must remain UNBOUND")
    require(evidence["execution_scope"]["implementation_ready"] is False, "must not claim readiness")

    vectors = load_json("INPUT-ONLY-VECTORS.json")
    require(vectors["vectors"][0]["expected"]["resolved_open_closure_writes"] == 0, "guard vector mismatch")
    require(vectors["vectors"][1]["expected"]["f12_bsi_u5_join"] == "UNBOUND", "ID12 vector mismatch")
    require(vectors["lineage"]["source_manifest_sha256"] == SOURCE_MANIFEST, "vector source mismatch")
    sources = load_json("SOURCE-MANIFESTS.json")
    require(sources["incoming_package"]["manifest_sha256"] == SOURCE_MANIFEST, "source binding mismatch")
    require(sources["prior_ddpi_id12_boundary"]["manifest_sha256"] == DDPI_SUPPLEMENT, "prior boundary mismatch")
    require(sources["copied_ghidra_trace_or_script"] is False, "raw trace/script must not be copied")
    print(f"PASS package={PACKAGE_ID} open_guard=ZERO_INIT_STATIC ID12_BSI=UNBOUND")


if __name__ == "__main__":
    if sys.argv[1:] == ["--build-manifest"]:
        build_manifest()
    elif not sys.argv[1:]:
        check()
    else:
        raise SystemExit("usage: python CHECK.py [--build-manifest]")
