from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "USER-DESIGNATED-KEYLESS-JOC-COMPARATOR-IDENTITY-CLARIFICATION-V1"
PAYLOADS = ("EVIDENCE.json", "INPUT-ONLY-VECTORS.json", "REPORT.md", "SOURCE-MANIFESTS.json")
SOURCE_PACKAGE = "0408ea2f104827371f808566c15c67525f889c7890b5253d90306eb1ba83ad7f"
SOURCE_SEAL = "796228267ac2b5260995a0fd3f4e7bfc897af3b6737c2c5d8c817e12e87739cb"
CROSSWALK = "c98042d10bece4eb3b8b8d148d8c76d16bf0f2a54a83cc58212cd984ecb7330b"
ADM_SHA256 = "7826dcbef37ef69642a6164c97a37bcfb3f917cea74aa0f0a30d76d0a825f1d1"
CANDIDATE_SHA256 = "383d7de4a57fd9bb32b8f871af1d2b5b0ad686a4b25e5fc6e51cd1be3adf6e86"
CANDIDATE_BYTES = 17395712


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
        "source_reverse_manifest_sha256": SOURCE_PACKAGE,
        "status": "COMPARATOR_HASH_AND_ADM_IDENTITY_BOUND; PYTHON_RUN_RECEIPT_NOT_RESEALED",
        "implementation_ready": False,
        "files": files,
    }
    raw = json.dumps(manifest, ensure_ascii=False, indent=2) + "\n"
    (ROOT / "MANIFEST.json").write_text(raw, encoding="utf-8", newline="\n")
    (ROOT / "MANIFEST.sha256").write_text(f"{sha256(ROOT / 'MANIFEST.json')}  MANIFEST.json\n", encoding="ascii", newline="\n")


def check() -> None:
    manifest = load_json("MANIFEST.json")
    require(manifest.get("package_id") == PACKAGE_ID, "package id mismatch")
    require(manifest.get("source_reverse_manifest_sha256") == SOURCE_PACKAGE, "source package hash mismatch")
    require(manifest.get("status") == "COMPARATOR_HASH_AND_ADM_IDENTITY_BOUND; PYTHON_RUN_RECEIPT_NOT_RESEALED", "status mismatch")
    require(manifest.get("implementation_ready") is False, "must not claim implementation readiness")
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
    require(evidence["source_wash_package"]["manifest_sha256"] == SOURCE_PACKAGE, "evidence source mismatch")
    require(evidence["source_wash_package"]["seal_sha256"] == SOURCE_SEAL, "evidence seal mismatch")
    require(evidence["comparator"]["sha256"] == CANDIDATE_SHA256, "candidate hash mismatch")
    require(evidence["comparator"]["bytes"] == CANDIDATE_BYTES, "candidate size mismatch")
    require(evidence["input_adm"]["sha256"] == ADM_SHA256, "ADM hash mismatch")
    require(evidence["input_adm"]["crosswalk_manifest_sha256"] == CROSSWALK, "crosswalk mismatch")
    require(evidence["input_adm"]["user_confirmed_same_as_python_prototype_original_adm"] is True, "user-confirmed input identity missing")
    require(evidence["python_run_receipt"]["status"] == "USER_ASSERTED_NOT_RESEALED", "run provenance status mismatch")
    require(evidence["python_run_receipt"]["blocks_exact_hash_comparison"] is False, "receipt absence must not block comparison")
    require(evidence["protected_dee_matching_candidate"] == "EXCLUDED_NOT_ACCESSED", "protected candidate scope mismatch")

    vector = load_json("INPUT-ONLY-VECTORS.json")["vectors"][0]
    require(vector["input"]["candidate_sha256"] == CANDIDATE_SHA256, "vector candidate mismatch")
    require(vector["input"]["candidate_bytes"] == CANDIDATE_BYTES, "vector size mismatch")
    require(vector["expected"]["acceptance_predicate"].startswith("generated_sha256 == candidate_sha256"), "acceptance predicate mismatch")
    require(vector["expected"]["application_exit_code"] == 0, "success exit code mismatch")

    sources = load_json("SOURCE-MANIFESTS.json")
    require(sources["source_comparator_identity_package"]["manifest_sha256"] == SOURCE_PACKAGE, "source binding mismatch")
    require(sources["authorized_adm_crosswalk"]["manifest_sha256"] == CROSSWALK, "crosswalk binding mismatch")
    require(sources["raw_audio_or_prototype_source_copied"] is False, "raw media/source must not be copied")
    print(f"PASS package={PACKAGE_ID} candidate={CANDIDATE_SHA256} input={ADM_SHA256} run_receipt=USER_ASSERTED_NOT_RESEALED")


if __name__ == "__main__":
    if sys.argv[1:] == ["--build-manifest"]:
        build_manifest()
    elif not sys.argv[1:]:
        check()
    else:
        raise SystemExit("usage: python CHECK.py [--build-manifest]")
