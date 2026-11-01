from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "ATTEMPT10-OUTPUT-PROFILE-RECONSTRUCTION-BOUNDARY-V1"
PAYLOADS = ("EVIDENCE.json", "INPUT-ONLY-VECTORS.json", "REPORT.md", "SOURCE-MANIFESTS.json")
PROFILE_MANIFEST = "91d2bfa7facc235c830f3916b784608fc7d45a7438dcf7643b31bf2b0f212054"
PROFILE_BINDING = "a4648c132cd87351a05ce39cb613c72b840e5f144c517768626ce13d6eb8ef1e"
BUILD_RECEIPT = "f173c95e01adf211e5bf07b73f9ee8f4f4080aae047ee138628dbb3aa109fc31"
RUN_RECEIPT_MANIFEST = "b1915d6de4d0eb7f05c8d2f14fb7f1eddbc368b6de4aeaf201ec37e3728eea91"
RUN_RECEIPT = "94dc3549d406e7ea572671498b8663ddb742c08f0cd0793880505837110dcb78"
CONFIG_AUDIT = "819227d2161c346b6a59d317ba3e98f9c2634e393726af8d4c9e074e7b31fec6"
COMPARATOR = "0504ef603cbe7008ba80c607d7d03f84c4ab9e7c12bd018716b82a3c954d7d67"
RUN_ID = "sample6-keyless-full-20261101-10"
OUTPUT_SHA = "593b6f866bf4396d25cd7b6307515f965f69e9884ce46599675c6f5d0a8c0473"
COMPARATOR_SHA = "383d7de4a57fd9bb32b8f871af1d2b5b0ad686a4b25e5fc6e51cd1be3adf6e86"
INPUT_SHA = "7826dcbef37ef69642a6164c97a37bcfb3f917cea74aa0f0a30d76d0a825f1d1"


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
        "source_reverse_manifest_sha256": PROFILE_MANIFEST,
        "status": "TARGET_PROFILE_SOURCE_SUPPORTED; RUN_CONFIG_RECONSTRUCTED_NOT_CAPTURED; HASH_MISMATCH",
        "implementation_ready": False,
        "files": files,
    }
    raw = json.dumps(manifest, ensure_ascii=False, indent=2) + "\n"
    (ROOT / "MANIFEST.json").write_text(raw, encoding="utf-8", newline="\n")
    (ROOT / "MANIFEST.sha256").write_text(f"{sha256(ROOT / 'MANIFEST.json')}  MANIFEST.json\n", encoding="ascii", newline="\n")


def check() -> None:
    manifest = load_json("MANIFEST.json")
    require(manifest.get("package_id") == PACKAGE_ID, "package id mismatch")
    require(manifest.get("source_reverse_manifest_sha256") == PROFILE_MANIFEST, "source manifest mismatch")
    require(manifest.get("status") == "TARGET_PROFILE_SOURCE_SUPPORTED; RUN_CONFIG_RECONSTRUCTED_NOT_CAPTURED; HASH_MISMATCH", "status mismatch")
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
    require(evidence["profile_source"]["manifest_sha256"] == PROFILE_MANIFEST, "profile source mismatch")
    require(evidence["profile_source"]["profile_binding_sha256"] == PROFILE_BINDING, "profile payload mismatch")
    require(evidence["profile_source"]["build_receipt_sha256"] == BUILD_RECEIPT, "build receipt mismatch")
    require(evidence["build_and_run"]["run_id"] == RUN_ID, "run id mismatch")
    require(evidence["build_and_run"]["input_sha256"] == INPUT_SHA, "input mismatch")
    require(evidence["run_result"]["output_sha256"] == OUTPUT_SHA, "output mismatch")
    require(evidence["run_result"]["comparator_sha256"] == COMPARATOR_SHA, "comparator mismatch")
    require(evidence["run_result"]["sha256_equal"] is False, "hash must remain mismatch")
    require(evidence["receipt_limits"]["target_profile_binding"] == "SOURCE_SUPPORTED_RECONSTRUCTION; RUN-SPECIFIC SOURCE LINK UNBOUND", "profile boundary mismatch")

    vectors = load_json("INPUT-ONLY-VECTORS.json")
    require(vectors["vectors"][0]["expected"]["attempt10_uses_this_exact_source_snapshot"] == "NOT_CAPTURED", "build provenance must remain not captured")
    require(vectors["vectors"][1]["expected"]["mvp_exact_hash"] == "FAIL", "MVP result mismatch")
    sources = load_json("SOURCE-MANIFESTS.json")
    require(sources["run_completion_and_hash_comparison"]["manifest_sha256"] == RUN_RECEIPT_MANIFEST, "run receipt binding mismatch")
    require(sources["run_completion_and_hash_comparison"]["receipt_sha256"] == RUN_RECEIPT, "run receipt payload mismatch")
    require(sources["run_config_audit"]["manifest_sha256"] == CONFIG_AUDIT, "config audit binding mismatch")
    require(sources["comparator_identity_manifest_sha256"] == COMPARATOR, "comparator binding mismatch")
    require(sources["raw_source_or_output_bytes_copied"] is False, "raw content must not be copied")
    print(f"PASS package={PACKAGE_ID} profile=SOURCE_SUPPORTED_RUN_LINK=UNBOUND exact_hash=MISMATCH")


if __name__ == "__main__":
    if sys.argv[1:] == ["--build-manifest"]:
        build_manifest()
    elif not sys.argv[1:]:
        check()
    else:
        raise SystemExit("usage: python CHECK.py [--build-manifest]")
