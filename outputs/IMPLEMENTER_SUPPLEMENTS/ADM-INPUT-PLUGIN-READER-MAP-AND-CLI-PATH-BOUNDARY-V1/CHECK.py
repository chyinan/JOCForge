from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "ADM-INPUT-PLUGIN-READER-MAP-AND-CLI-PATH-BOUNDARY-V1"
PAYLOADS = ("EVIDENCE.json", "INPUT-ONLY-VECTORS.json", "REPORT.md", "SOURCE-MANIFESTS.json")
SOURCE_MANIFEST = "abadcde4fbae11a5babf18cf136873d5673b8d7ad9f9c8da3ff843f5fc70e1cd"
DEE_SHA256 = "65485b6d35fa534b25109ab6f238971aaded73fc99c7f7fb119d1a5d5299234e"
DDP_SHA256 = "2fd503793f68dfa13037adf4082b8e5327e5046abfb6183ee84d7ae5839ecd72"
ATMOS_SHA256 = "3d66bcec36031fd48e6565d15f05fea656642377ca4f8c98cdce1cce8b7e95d2"
SINGLE_PASS_SHA256 = "1767070831dc0d5925a1fc401684ae1447cef2d6bc59659f8a25f53c95f482e9"
ADM_SHA256 = "7826dcbef37ef69642a6164c97a37bcfb3f917cea74aa0f0a30d76d0a825f1d1"


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
        "status": "STATIC_ADM_FACTORY_MAP_BOUND; CLI_OWNER_AND_RUNTIME_READ_UNBOUND",
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
    require(manifest.get("status") == "STATIC_ADM_FACTORY_MAP_BOUND; CLI_OWNER_AND_RUNTIME_READ_UNBOUND", "status mismatch")
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
    image_hashes = {item["sha256"] for item in evidence["target_images"]}
    require(image_hashes == {DEE_SHA256, DDP_SHA256, ATMOS_SHA256, SINGLE_PASS_SHA256}, "target image set mismatch")
    require(evidence["adm_factory"]["type_label"] == "adm", "ADM type label mismatch")
    require(evidence["adm_factory"]["concrete_reader"] == "dee::input::AtmosMezz", "reader mapping mismatch")
    require(evidence["cli_audio_path"]["job_config_owner_offset"] == "0x7D0", "audio list field mismatch")
    require(evidence["cli_audio_path"]["selected_reader_setter_owner_join"] == "UNBOUND", "owner join must remain UNBOUND")
    require(evidence["derived_pre_run_observation"]["input_sha256"] == ADM_SHA256, "derived input hash mismatch")
    require(evidence["derived_pre_run_observation"]["run_called"] is False, "Run must remain not called")
    require(evidence["derived_pre_run_observation"]["original_r138_acceptance"] is False, "do not claim original profile acceptance")
    require(evidence["restrictions"]["implementation_ready"] is False, "evidence must not claim readiness")

    vectors = load_json("INPUT-ONLY-VECTORS.json")
    require(vectors["vectors"][0]["expected"]["concrete_reader"] == "dee::input::AtmosMezz", "vector reader mapping mismatch")
    require(vectors["vectors"][1]["expected"]["selected_reader_setter_owner"] == "UNBOUND", "vector must preserve owner boundary")
    require(vectors["vectors"][2]["input"]["input_sha256"] == ADM_SHA256, "vector input hash mismatch")
    require(vectors["lineage"]["source_manifest_sha256"] == SOURCE_MANIFEST, "vector lineage mismatch")

    sources = load_json("SOURCE-MANIFESTS.json")
    require(sources["incoming_package"]["manifest_sha256"] == SOURCE_MANIFEST, "source binding mismatch")
    require(sources["copied_raw_trace_or_script"] is False, "raw trace/script must not be copied")
    print(f"PASS package={PACKAGE_ID} adm_reader=AtmosMezz cli_owner_join=UNBOUND run=NOT_RUN")


if __name__ == "__main__":
    if sys.argv[1:] == ["--build-manifest"]:
        build_manifest()
    elif not sys.argv[1:]:
        check()
    else:
        raise SystemExit("usage: python CHECK.py [--build-manifest]")
