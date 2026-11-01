from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "F15-FINAL-EXPOSURE-TERMINAL-CONTRACT-V1"
PAYLOADS = ("EVIDENCE.json", "INPUT-ONLY-VECTORS.json", "REPORT.md", "SOURCE-MANIFESTS.json")
SOURCE_MANIFEST = "d96bf37326e454e6d4726276a00f72dd99a1c2b2148eb81070c1d02439c2f0ce"
ATTEMPT08_MANIFEST = "c4f855a6c4830c8f3f1dba472def2e62ea710b73b122670ffd644ca9ceaa5c56"
SCHEMA_MANIFEST = "d8d97f07d82d07dfa05772bb60bc90c6cab46e908f04366f24cc1398a16d169f"


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
        "status": "F15_TERMINAL_CONTRACT_BOUND; SERIALIZED_RECEIPT_SCHEMA_UNBOUND",
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
    require(manifest.get("status") == "F15_TERMINAL_CONTRACT_BOUND; SERIALIZED_RECEIPT_SCHEMA_UNBOUND", "status mismatch")
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
    require(evidence["source"]["manifest_sha256"] == SOURCE_MANIFEST, "source package mismatch")
    require(evidence["attempt_08"]["source_eof_ledger"] == "PASS_WITHIN_LOGGED_SCOPE", "EOF status mismatch")
    require(evidence["attempt_08"]["application_exit_code"] == 2, "attempt exit mismatch")
    require(evidence["attempt_08"]["final_exposure_receipt"] == "NOT_CAPTURED", "receipt status mismatch")
    require(evidence["attempt_08"]["exact_hash_comparison_eligible"] is False, "failed output must be ineligible")
    require(evidence["missing_schema_and_repair"]["serialized_field_names"] == "UNBOUND", "schema must remain UNBOUND")
    require(evidence["execution_scope"]["implementation_ready"] is False, "must not claim readiness")

    vectors = load_json("INPUT-ONLY-VECTORS.json")
    require(vectors["vectors"][0]["expected"]["DONE"] == "NOT_REACHED", "failure vector mismatch")
    require(vectors["vectors"][0]["expected"]["exact_hash_comparison_eligible"] is False, "failure must be ineligible")
    require(vectors["lineage"]["source_manifest_sha256"] == SOURCE_MANIFEST, "vector source mismatch")
    sources = load_json("SOURCE-MANIFESTS.json")
    require(sources["incoming_package"]["manifest_sha256"] == SOURCE_MANIFEST, "source binding mismatch")
    require(sources["attempt_08_receipt_source"]["manifest_sha256"] == ATTEMPT08_MANIFEST, "attempt receipt mismatch")
    require(sources["referenced_unavailable_receipt_schema"]["manifest_sha256"] == SCHEMA_MANIFEST, "schema hash mismatch")
    require(sources["raw_audio_or_codec_material_copied"] is False, "raw material must not be copied")
    print(f"PASS package={PACKAGE_ID} attempt08=POST_FAILURE schema_fields=UNBOUND")


if __name__ == "__main__":
    if sys.argv[1:] == ["--build-manifest"]:
        build_manifest()
    elif not sys.argv[1:]:
        check()
    else:
        raise SystemExit("usage: python CHECK.py [--build-manifest]")
