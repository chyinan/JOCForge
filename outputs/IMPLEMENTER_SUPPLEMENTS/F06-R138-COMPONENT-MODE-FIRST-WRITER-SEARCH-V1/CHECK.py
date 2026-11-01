from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "F06-R138-COMPONENT-MODE-FIRST-WRITER-SEARCH-V1"
PAYLOADS = ("EVIDENCE.json", "INPUT-ONLY-VECTORS.json", "REPORT.md", "SOURCE-MANIFESTS.json")
SOURCE_MANIFEST = "44cb132c8623693e89cb612c59d42479cf38708a7a81f0f438b231c29ab905df"
JOB_PARENT = "ba95fbedd155591bddf10eb11a7b21f3dc3c5dab414b52627c3a1e5c8dcbf74c"
OWNER_PARENT = "54ac629045a8d5b348f43f993894bbd9f0a707e6f165ff91d8e946ea41505810"
IMAGE_HASH = "3d66bcec36031fd48e6565d15f05fea656642377ca4f8c98cdce1cce8b7e95d2"


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
        "status": "PLUS_0X54_SCAN_BOUND; COMPONENT_OWNER_AND_R138_TRANSFER_UNBOUND",
        "implementation_ready": False,
        "files": files,
    }
    raw = json.dumps(manifest, ensure_ascii=False, indent=2) + "\n"
    (ROOT / "MANIFEST.json").write_text(raw, encoding="utf-8", newline="\n")
    (ROOT / "MANIFEST.sha256").write_text(f"{sha256(ROOT / 'MANIFEST.json')}  MANIFEST.json\n", encoding="ascii", newline="\n")


def check() -> None:
    manifest = load_json("MANIFEST.json")
    require(manifest.get("package_id") == PACKAGE_ID, "package id mismatch")
    require(manifest.get("source_reverse_manifest_sha256") == SOURCE_MANIFEST, "source mismatch")
    require(manifest.get("status") == "PLUS_0X54_SCAN_BOUND; COMPONENT_OWNER_AND_R138_TRANSFER_UNBOUND", "status mismatch")
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
    require(evidence["source"]["target_image_sha256"] == IMAGE_HASH, "image mismatch")
    require(evidence["parents"][0]["manifest_sha256"] == JOB_PARENT, "job parent mismatch")
    require(evidence["parents"][1]["manifest_sha256"] == OWNER_PARENT, "owner parent mismatch")
    require(evidence["scan"]["plus_0x54_writes"] == 243, "write count mismatch")
    require(evidence["scan"]["target_component_callback_writes"] == 0, "callback write count mismatch")
    require(evidence["scan"]["target_component_joined_to_whole_image_candidate"] is False, "candidate join must remain false")
    require(evidence["rejected_candidate"]["included_as_target_writer"] is False, "stack-local candidate must remain excluded")
    require(evidence["conclusion"]["component_instance_first_writer"] == "UNBOUND", "component writer must remain UNBOUND")
    require(evidence["conclusion"]["global_no_writer_claim"] is False, "global no-writer claim must remain false")
    require(evidence["scope_deviation"]["excluded_from_closure_claim"] is True, "scope deviation must be disclosed")

    vectors = load_json("INPUT-ONLY-VECTORS.json")
    require(vectors["vectors"][0]["expected"]["component_first_writer"] == "UNBOUND", "vector status mismatch")
    require(vectors["lineage"]["source_manifest_sha256"] == SOURCE_MANIFEST, "vector source mismatch")
    require(vectors["lineage"]["target_image_sha256"] == IMAGE_HASH, "vector image mismatch")
    sources = load_json("SOURCE-MANIFESTS.json")
    require(sources["incoming_package"]["manifest_sha256"] == SOURCE_MANIFEST, "source binding mismatch")
    require(sources["scope_deviation_disclosed"] is True, "scope deviation missing")
    require(sources["copied_raw_analysis_or_media"] is False, "raw material must not be copied")
    print(f"PASS package={PACKAGE_ID} explicit_writes=243 target_join=UNBOUND scope_deviation=DISCLOSED")


if __name__ == "__main__":
    if sys.argv[1:] == ["--build-manifest"]:
        build_manifest()
    elif not sys.argv[1:]:
        check()
    else:
        raise SystemExit("usage: python CHECK.py [--build-manifest]")
