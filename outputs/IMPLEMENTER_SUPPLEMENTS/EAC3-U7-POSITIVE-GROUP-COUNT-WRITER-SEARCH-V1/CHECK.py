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
require(manifest.get("package_id") == "EAC3-U7-POSITIVE-GROUP-COUNT-WRITER-SEARCH-V1", "package id")
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
require(evidence["source_package"]["manifest_sha256"] == "d279a5cba1dda44b0265dec0eebe0bae4b6c25ec95135fba72a54fcc28ca131c", "source manifest binding")
require(evidence["field_contract"]["n_field"] == "u16(state+0x141c8+2*j)", "N field")
require(evidence["search_boundary"]["positive_n_writer"] == "UNBOUND", "positive N writer remains open")
require(evidence["best_supported_model_candidate"]["grade"] == "INFERENCE_ONLY", "candidate grade")
d = vector["difference_count_d"]
n = (d + 2) // 3
require(vector["candidate_group_count_n"] == n, "candidate N arithmetic")
require(vector["candidate_tail_values"] == 3 * n - d, "candidate tail arithmetic")
require(vector["native_n_assignment_observed"] is False and vector["runtime_execution"] == "NOT_RUN", "no native runtime claim")
require(evidence["execution_boundary"]["media_pcm_payload_or_key_read"] is False, "media/key boundary")

print("PASS: focused positive-N search scope, inference-only vector, and NOT_RUN boundary")
