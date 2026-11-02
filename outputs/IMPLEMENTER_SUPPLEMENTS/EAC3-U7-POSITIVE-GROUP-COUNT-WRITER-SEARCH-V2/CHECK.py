from __future__ import annotations

import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent


def require(condition: bool, label: str) -> None:
    if not condition:
        raise SystemExit(f"FAIL {label}")


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


manifest_path = ROOT / "MANIFEST.json"
manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
require(sha256(manifest_path) == (ROOT / "MANIFEST.sha256").read_text(encoding="ascii").split()[0], "manifest hash")
require(manifest.get("package_id") == "EAC3-U7-POSITIVE-GROUP-COUNT-WRITER-SEARCH-V2", "package id")
payload = manifest.get("files")
require(isinstance(payload, list) and len(payload) == 4, "payload inventory")
actual_names = {path.name for path in ROOT.iterdir() if path.is_file() and path.name not in {"MANIFEST.json", "MANIFEST.sha256", "package_seal.json"}}
require(actual_names == {entry.get("file") for entry in payload}, "payload file set")
for entry in payload:
    path = ROOT / entry["file"]
    require(path.stat().st_size == entry["bytes"], f"size {entry['file']}")
    require(sha256(path) == entry["sha256"], f"hash {entry['file']}")

seal = json.loads((ROOT / "package_seal.json").read_text(encoding="utf-8"))
sealed = seal.get("files")
require(isinstance(sealed, list) and seal.get("file_count") == len(sealed), "seal inventory")
require({entry.get("file") for entry in sealed} == actual_names | {"MANIFEST.json", "MANIFEST.sha256"}, "sealed file set")
for entry in sealed:
    path = ROOT / entry["file"]
    require(path.stat().st_size == entry["bytes"], f"sealed size {entry['file']}")
    require(sha256(path) == entry["sha256"], f"sealed hash {entry['file']}")

evidence = json.loads((ROOT / "EVIDENCE.json").read_text(encoding="utf-8"))
vector = json.loads((ROOT / "INPUT-ONLY-VECTOR.json").read_text(encoding="utf-8"))
require(evidence["source_package"]["manifest_sha256"] == "eea5ab0697decd3536cf35330f7bf0bf7036d1888a2471a8300518734ea5db0f", "source manifest binding")
require(evidence["inventory"]["loaded_instruction_count"] == 1276863, "full image instruction count")
require(evidence["inventory"]["literal_field_displacement_0x141c8_occurrences"] == 0, "field displacement count")
require(evidence["inventory"]["state_subobject_base_0x140f0_lea_occurrences"] == 21, "base reference count")
require(evidence["field_and_candidate_rule"]["positive_native_n_writer"] == "UNBOUND", "positive writer remains unbound")
require(evidence["field_and_candidate_rule"]["candidate_grade"] == "INFERENCE_ONLY", "candidate grade")
d = vector["difference_count_d"]
n = (d + 2) // 3
require(vector["candidate_group_count_n"] == n == 2, "candidate N vector")
require(vector["tail_values"] == 3 * n - d == 2, "candidate tail vector")
require(vector["native_n_assignment_observed"] is False and vector["runtime_execution"] == "NOT_RUN", "native boundary")
require(evidence["execution_boundary"]["media_pcm_payload_or_key_read"] is False, "media and key boundary")

print("PASS: full-image N reference inventory, inference-only vector, and runtime boundary")
