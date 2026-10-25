from __future__ import annotations

import hashlib
import json
import math
import struct
from pathlib import Path


ROOT = Path(__file__).resolve().parent


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


def read_json(name: str) -> dict:
    return json.loads((ROOT / name).read_text(encoding="utf-8"))


def f32(value: float) -> float:
    return struct.unpack("<f", struct.pack("<f", value))[0]


def check_files() -> int:
    manifest_bytes = (ROOT / "MANIFEST.json").read_bytes()
    manifest = json.loads(manifest_bytes.decode("utf-8"))
    entries = manifest["files"]
    expected = {entry["path"] for entry in entries} | {"MANIFEST.json", "DELIVERY-ENVELOPE.json"}
    actual = {path.name for path in ROOT.iterdir() if path.is_file()}
    require(actual == expected, "file set mismatch")
    for entry in entries:
        path = ROOT / entry["path"]
        require(path.stat().st_size == entry["bytes"], f"byte count mismatch: {entry['path']}")
        require(hashlib.sha256(path.read_bytes()).hexdigest() == entry["sha256"], f"hash mismatch: {entry['path']}")
    envelope = read_json("DELIVERY-ENVELOPE.json")
    require(envelope["manifest_sha256"] == hashlib.sha256(manifest_bytes).hexdigest(), "manifest seal mismatch")
    return len(entries)


def check_prefix() -> None:
    table = read_json("FIELD-TABLE.json")
    fields = table["fields"]
    require(len(fields) == 9, "selected prefix field count mismatch")
    cursor = 0
    bit_string = ""
    for field in fields:
        width = field["width_bits"]
        value = field["value"]
        require(field["bit_start"] == cursor, f"non-contiguous field: {field['name']}")
        require(field["bit_end_inclusive"] == cursor + width - 1, f"field extent mismatch: {field['name']}")
        require(0 <= value < (1 << width), f"value does not fit: {field['name']}")
        bit_string += format(value, f"0{width}b")
        cursor += width
    vectors = read_json("INPUT-ONLY-VECTORS.json")
    expected = vectors["vectors"][0]["expected"]
    require(cursor == expected["bit_count"] == 45, "prefix width mismatch")
    require(bit_string == expected["bit_string"] == table["recomputed_prefix_bits"], "prefix bits mismatch")
    require(expected["display_byte_hex"] == table["display_only_padded_byte_hex"], "display byte mismatch")
    require(expected["display_pad_bits"] == "000" and expected["pad_bits_are_not_prefix"], "display pad scope mismatch")


def check_dialnorm() -> None:
    vectors = read_json("INPUT-ONLY-VECTORS.json")["vectors"]
    for vector in vectors[1:]:
        case = vector["input"]
        if "selected_lufs_f32_word" in case:
            word = int(case["selected_lufs_f32_word"], 16)
            lufs = struct.unpack(">f", word.to_bytes(4, "big"))[0]
        else:
            lufs = f32(case["selected_lufs_f32_decimal"])
        if abs(f32(lufs - f32(-120.0))) < 0.000001:
            dialnorm = 31
        else:
            value = math.trunc(f32(lufs - f32(0.5)))
            dialnorm = -max(-31, min(-1, value))
        expected = vector["expected"]
        require(dialnorm == expected["dialnorm"], f"dialnorm mismatch: {vector['id']}")
        require(format(dialnorm, "05b") == expected["dialnorm_five_bits"], f"wire bits mismatch: {vector['id']}")
        if "f06nd_n_f32_decimal" in expected:
            n = f32((-256 * dialnorm) * (2.0 ** -15))
            n_word = f"0x{int.from_bytes(struct.pack('>f', n), 'big'):08X}"
            require(n == expected["f06nd_n_f32_decimal"], "F06ND n value mismatch")
            require(n_word == expected["f06nd_n_f32_word"], "F06ND n word mismatch")
    status = read_json("DIALNORM-STATUS.json")
    require(status["formula_status"] == "STATIC_CLOSED", "formula status mismatch")
    require(status["typed_programme_source_join_status"] == "NOT_READY", "identity join was overstated")
    require(read_json("EVIDENCE.json")["provenance"]["raw_pcm_or_frame_payload_included"] is False, "raw media scope mismatch")


def main() -> None:
    count = check_files()
    check_prefix()
    check_dialnorm()
    print(f"PASS_F12_PREFIX_DIALNORM_WASH_PACKAGE files={count} formula_and_vectors=PASS identity_join=NOT_READY")


if __name__ == "__main__":
    main()
