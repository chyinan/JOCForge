import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent


def require(condition, message):
    if not condition:
        raise SystemExit(message)


manifest_path = ROOT / "MANIFEST.json"
manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
seal = json.loads((ROOT / "package_seal.json").read_text(encoding="utf-8"))
manifest_hash = hashlib.sha256(manifest_path.read_bytes()).hexdigest()
require(manifest["package_id"] == "JOCFORGE-EAC3-CHANMAP-U16-SERIALIZATION-V1", "package id mismatch")
require(manifest["schema"] == "wash-sanitized-supplement.v1", "manifest schema mismatch")
require(seal["manifest_sha256"] == manifest_hash, "seal does not bind manifest")
require((ROOT / "MANIFEST.sha256").read_text(encoding="utf-8").split()[0] == manifest_hash, "manifest checksum mismatch")

expected = {item["path"] for item in manifest["files"]}
actual = {path.name for path in ROOT.iterdir() if path.is_file()} - {"MANIFEST.json", "MANIFEST.sha256", "package_seal.json"}
require(actual == expected, "payload set mismatch")
for item in manifest["files"]:
    payload = ROOT / item["path"]
    require(payload.stat().st_size == item["bytes"], f"size mismatch: {item['path']}")
    require(hashlib.sha256(payload.read_bytes()).hexdigest() == item["sha256"], f"hash mismatch: {item['path']}")

vectors = json.loads((ROOT / "INPUT-ONLY-VECTORS.json").read_text(encoding="utf-8"))
require(vectors["status"] == "INPUT_ONLY_NOT_RUN", "vector status mismatch")
require(len(vectors["one_hot_vectors"]) == 16, "expected 16 one-hot vectors")
for vector in vectors["one_hot_vectors"]:
    index = vector["numeric_bit_index"]
    mask = int(vector["mask_hex"], 16)
    require(mask == 1 << index, f"one-hot mask mismatch: {index}")
    require(vector["expected_emitted_slot"] == 15 - index, f"slot mismatch: {index}")
    require(vector["expected_field_bits"] == format(mask, "016b"), f"field bits mismatch: {index}")

target = vectors["target_composite"]
require(target["mask_hex"] == "0x1A00", "target mask mismatch")
require(target["set_bits_lsb_indexed"] == [12, 11, 9], "target set bits mismatch")
require(target["expected_emitted_slots"] == [3, 4, 6], "target emitted slots mismatch")
require(target["expected_field_bits"] == "0001101000000000", "target field bits mismatch")
require(target["ADM_source_role_join"] == "UNBOUND", "ADM source-role boundary changed")
require(target["native_execution"] == "NOT_RUN", "native execution boundary changed")
require(seal["production_allowed"] is False, "production gate changed")
print("PASS: manifest, seal, 16 one-hot vectors, target mask, and Wash boundaries")
