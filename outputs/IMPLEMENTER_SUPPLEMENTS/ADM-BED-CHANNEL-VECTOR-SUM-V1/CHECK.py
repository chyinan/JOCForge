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
require(manifest.get("package_id") == "ADM-BED-CHANNEL-VECTOR-SUM-V1", "package id")
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
require(evidence["source_package"]["manifest_sha256"] == "bc0460ba82a4211010881becd3a5e3744aedede7b97042c7998919242b5c9dee", "source manifest binding")
rows = evidence["bed_channel_vectors"]
require(len(rows) == 10 and [row["track"] for row in rows] == list(range(1, 11)), "bed row inventory")
require(all(row["vector_length"] == 1 for row in rows), "one channel per metadata bed row")
require(sum(row["vector_length"] for row in rows) == vector["bed_vector_sum"] == 10, "bed vector sum")
require(evidence["derived_candidate"]["object_source_count"] + vector["bed_vector_sum"] == vector["candidate_master_channel_count"] == 38, "candidate count")
require(evidence["derived_candidate"]["native_storage_vector_identity"] == "UNBOUND", "native runtime boundary")
require(evidence["execution_boundary"]["dee_run"] == "NOT_RUN" and evidence["execution_boundary"]["private_key_or_license_material_accessed"] is False, "execution/secret boundary")

print("PASS: 10 one-channel AXML/CHNA bed vectors plus 28 object rows yield a 38-channel metadata candidate")
