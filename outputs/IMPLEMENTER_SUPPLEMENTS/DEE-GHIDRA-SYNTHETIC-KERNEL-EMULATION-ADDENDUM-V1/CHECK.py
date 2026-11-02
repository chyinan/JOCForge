from __future__ import annotations

import hashlib
import json
import struct
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
require(manifest.get("package_id") == "DEE-GHIDRA-SYNTHETIC-KERNEL-EMULATION-ADDENDUM-V1", "package id")
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
vectors = json.loads((ROOT / "INPUT-VECTORS.json").read_text(encoding="utf-8"))
require(evidence["source_package"]["manifest_sha256"] == "f3b19c63792632306bdd161dea639df6c7e21e890ba209f0c0f15ff3caf518e9", "source manifest binding")
require(evidence["source_package"]["target_image_sha256"] == "3d66bcec36031fd48e6565d15f05fea656642377ca4f8c98cdce1cce8b7e95d2", "target identity")
require(len(vectors["cases"]) == 2 and vectors["vectors_are_audio"] is False, "synthetic vectors only")

delta = evidence["isolated_calls"][0]
delta_input = delta["input"]
previous = delta_input["previous_s16"]
valid = []
for current in delta_input["current_s16"][:delta_input["valid_count"]]:
    valid.append(current - previous + 2)
    previous = current
tail_count = 3 * delta_input["total_slots"] - delta_input["valid_count"]
expected_s16 = [delta_input["previous_s16"], *valid, *([2] * tail_count)]
expected_bytes = b"".join(struct.pack("<h", value) for value in expected_s16[1:])
require(expected_s16 == delta["observed"]["result_s16"], "delta-plus-two and tail-fill vector")
require(expected_bytes.hex() == delta["observed"]["output_le_hex"], "delta output bytes")
require(hashlib.sha256(expected_bytes).hexdigest() == delta["observed"]["output_sha256"], "delta output digest")

writer = evidence["isolated_calls"][1]
bits = "".join(format(value, f"0{writer['input']['width_bits']}b") for value in writer["input"]["unsigned_values"])
word_bits, residual_bits = bits[:16], bits[16:]
word = int(word_bits, 2)
residual = int(residual_bits.ljust(16, "0"), 2) if residual_bits else 0
require(f"0x{word:04x}" == writer["observed"]["completed_word"], "writer completed word")
require(struct.pack("<H", word).hex() == writer["observed"]["output_memory_le_hex"], "writer memory byte order")
require(len(residual_bits) == writer["observed"]["remaining_cursor_bits"], "writer residual cursor")
require(f"0x{residual:04x}" == writer["observed"]["remaining_accumulator"], "writer residual accumulator")
require(hashlib.sha256(bytes.fromhex(writer["observed"]["output_memory_le_hex"])).hexdigest() == writer["observed"]["output_sha256"], "writer output digest")

boundaries = evidence["execution_boundaries"]
for key in ("dll_loaded_by_windows", "full_encoder_called", "adm_or_media_read", "pcm_or_encoded_payload_read", "product_audio_generated", "key_or_license_read", "synthetic_bytes_are_product_audio", "harness_or_inspection_source_copied", "target_binary_or_decompilation_copied"):
    require(boundaries[key] is False, f"boundary {key}")
require(evidence["integration_status"]["complete_frame_or_au_validation"] == "NOT_RUN", "AU validation remains not run")

print("PASS: hash-bound synthetic delta/tail and bit-writer vectors; product-audio boundary preserved")
