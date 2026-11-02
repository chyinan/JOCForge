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
require(manifest.get("package_id") == "EAC3-U7-GROUP-COUNT-RESET-AND-CANDIDATE-V1", "package id")
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
vectors = json.loads((ROOT / "INPUT-ONLY-VECTORS.json").read_text(encoding="utf-8"))
require(evidence["field_contract"]["n_type_and_expression"] == "u16(state + 0x141c8 + 2*j)", "N field")
require(evidence["field_contract"]["d_type_and_expression"] == "i16(state + 0x1418c + 10*block + 2*lane)", "D field")
require(evidence["typed_reset_candidate"]["role"] == "zero initialization candidate; not a positive runtime N producer", "reset scope")
require(evidence["typed_reset_candidate"]["same_instance_as_frame_writer"] == "UNBOUND", "instance join")
require(evidence["bounded_scan"]["positive_n_writer"] == "UNBOUND", "positive producer")
require(evidence["model_candidate"]["grade"] == "INFERENCE_ONLY", "candidate grade")
require(vectors["native_assignment_observed"] is False and len(vectors["vectors"]) == 5, "input-only vectors")
for row in vectors["vectors"]:
    d = row["D"]
    n = (d + 2) // 3
    require(row["N"] == n, f"N vector D={d}")
    require(row["tail_count"] == 3 * n - d and 0 <= row["tail_count"] <= 2, f"tail vector D={d}")
require(evidence["execution_boundary"]["native_runtime"] == "NOT_RUN", "runtime status")
require(evidence["execution_boundary"]["key_or_auth_material_read"] is False, "key boundary")

print("PASS: hash-bound D/N reset, input-only ceil candidate vectors, and UNBOUND runtime ownership")
