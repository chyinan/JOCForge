from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "ATTEMPT10-KEYLESS-MVP-MISMATCH-AND-PROFILE-BOUNDARY-V1"
PAYLOADS = ("EVIDENCE.json", "INPUT-ONLY-VECTORS.json", "REPORT.md", "SOURCE-MANIFESTS.json")
RUN_MANIFEST = "b1915d6de4d0eb7f05c8d2f14fb7f1eddbc368b6de4aeaf201ec37e3728eea91"
RUN_RECEIPT = "94dc3549d406e7ea572671498b8663ddb742c08f0cd0793880505837110dcb78"
HASH_CHECK = "211afb5a35dfad179d62860f672839b8c6d52a96e2cb660981a9e747bf8603fb"
CONFIG_AUDIT = "819227d2161c346b6a59d317ba3e98f9c2634e393726af8d4c9e074e7b31fec6"
COMPARATOR_PACKAGE = "0504ef603cbe7008ba80c607d7d03f84c4ab9e7c12bd018716b82a3c954d7d67"
INPUT_SHA = "7826dcbef37ef69642a6164c97a37bcfb3f917cea74aa0f0a30d76d0a825f1d1"
OUTPUT_SHA = "593b6f866bf4396d25cd7b6307515f965f69e9884ce46599675c6f5d0a8c0473"
COMPARATOR_SHA = "383d7de4a57fd9bb32b8f871af1d2b5b0ad686a4b25e5fc6e51cd1be3adf6e86"
FILE_BYTES = 17395712


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
        "source_reverse_manifest_sha256": RUN_MANIFEST,
        "status": "DONE_EXIT0_HASH_MISMATCH; TARGET_OUTPUT_PROFILE_UNBOUND",
        "implementation_ready": False,
        "files": files,
    }
    raw = json.dumps(manifest, ensure_ascii=False, indent=2) + "\n"
    (ROOT / "MANIFEST.json").write_text(raw, encoding="utf-8", newline="\n")
    (ROOT / "MANIFEST.sha256").write_text(f"{sha256(ROOT / 'MANIFEST.json')}  MANIFEST.json\n", encoding="ascii", newline="\n")


def check() -> None:
    manifest = load_json("MANIFEST.json")
    require(manifest.get("package_id") == PACKAGE_ID, "package id mismatch")
    require(manifest.get("source_reverse_manifest_sha256") == RUN_MANIFEST, "source manifest mismatch")
    require(manifest.get("status") == "DONE_EXIT0_HASH_MISMATCH; TARGET_OUTPUT_PROFILE_UNBOUND", "status mismatch")
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
    require(evidence["run"]["run_id"] == "sample6-keyless-full-20261101-10", "run id mismatch")
    require(evidence["run"]["input_sha256"] == INPUT_SHA, "input mismatch")
    require(evidence["run"]["application_status"] == "DONE" and evidence["run"]["exit_code"] == 0, "run terminal mismatch")
    require(evidence["output"]["sha256"] == OUTPUT_SHA and evidence["output"]["bytes"] == FILE_BYTES, "output identity mismatch")
    require(evidence["comparator"]["sha256"] == COMPARATOR_SHA and evidence["comparator"]["bytes"] == FILE_BYTES, "comparator identity mismatch")
    require(evidence["comparison"]["sha256_equal"] is False and evidence["comparison"]["root_cause"] == "NOT_INFERRED", "mismatch result mismatch")
    require(evidence["output_profile"]["output_codec_profile"] == "UNBOUND", "profile must remain UNBOUND")
    require(evidence["terminal_receipt"]["structured_final_exposure_receipt"] == "NOT_PRESENT_AS_TYPED_OBJECT_IN_PERSISTED_RESULT", "receipt boundary mismatch")

    sources = load_json("SOURCE-MANIFESTS.json")
    require(sources["run_receipt_package"]["manifest_sha256"] == RUN_MANIFEST, "run source binding mismatch")
    require(sources["run_receipt_package"]["receipt_sha256"] == RUN_RECEIPT, "receipt hash mismatch")
    require(sources["independent_hash_comparison"]["manifest_sha256"] == HASH_CHECK, "hash check source mismatch")
    require(sources["run_configuration_audit"]["manifest_sha256"] == CONFIG_AUDIT, "config audit source mismatch")
    require(sources["comparator_identity"]["manifest_sha256"] == COMPARATOR_PACKAGE, "comparator source mismatch")
    require(sources["raw_output_or_comparator_payload_copied"] is False, "raw output must not be copied")
    print(f"PASS package={PACKAGE_ID} result=MISMATCH target_profile=UNBOUND")


if __name__ == "__main__":
    if sys.argv[1:] == ["--build-manifest"]:
        build_manifest()
    elif not sys.argv[1:]:
        check()
    else:
        raise SystemExit("usage: python CHECK.py [--build-manifest]")
