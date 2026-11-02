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
require(manifest.get("package_id") == "EAC3-EXPONENT-DIFFERENCE-COUNT-CANDIDATE-V1", "package id")
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
require(evidence["source_package"]["manifest_sha256"] == "d76ea8381fc3d799bfea3b770641d0104a5297681de664debd6d10721ece6304", "source manifest binding")
require(evidence["field_contract"]["d_expression"] == "state+0x1418c+10*block+2*lane", "D field")
require(evidence["producer_search"]["new_positive_d_writer"] == "UNBOUND", "positive D writer remains open")
require(evidence["best_supported_model_candidate"]["grade"] == "INFERENCE_ONLY", "D rule remains inference")
row = evidence["combined_input_only_vector"]
vector_row = vector
indices = [0] + [1 + i * vector_row["stride_in_short_entries"] for i in range(len(vector_row["selected_exponents"]) - 1)]
selected = [vector_row["row_values"][i] for i in indices]
require(indices == vector_row["selected_indices"] == [0, 1, 3, 5, 7], "selected stride indices")
require(selected == vector_row["selected_exponents"] == [10, 11, 9, 8, 6], "selected exponent values")
require(vector_row["K_selected_samples_including_seed"] == len(selected) == 5, "selected sample count")
require(vector_row["D_candidate"] == len(selected) - 1 == 4, "D candidate arithmetic")
require(vector_row["N_candidate"] == (vector_row["D_candidate"] + 2) // 3 == 2, "N candidate arithmetic")
require(vector_row["tail_values_candidate"] == 3 * vector_row["N_candidate"] - vector_row["D_candidate"] == 2, "tail candidate arithmetic")
require(evidence["execution_boundary"]["native_runtime"] == "NOT_RUN", "runtime boundary")

print("PASS: selected-stride D/N inference vector and explicit native NOT_RUN boundary")
