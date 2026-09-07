# pattern: Imperative Shell

"""Create transient codecdatae-only variants of an existing I0/skipfld stream.

This diagnostic changes only the two codecdatae fields in payloads 11 and 14.
It removes the conditional eight-bit codec-data value when a flag changes from
one to zero, adjusts the EMDF and skip-field lengths, preserves the remaining
audio syntax, and rewrites E-AC-3 CRC2. It is not a public JOCForge encode
mode.
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path


class BitCursor:
    """Bounded MSB-first cursor over one packed bit span."""

    def __init__(self, bits: list[int], position: int = 0) -> None:
        self.bits = bits
        self.position = position

    def read(self, width: int) -> int:
        if width < 0 or self.position + width > len(self.bits):
            raise ValueError(f"bit read outside bounded input: {self.position}+{width}")
        value = 0
        for _ in range(width):
            value = (value << 1) | self.bits[self.position]
            self.position += 1
        return value


def variable_value(cursor: BitCursor, width: int, max_groups: int = 4) -> int:
    value = 0
    for _ in range(max_groups):
        value += cursor.read(width)
        if not cursor.read(1):
            return value
        value = (value << width) + (1 << width)
    raise ValueError("unterminated EMDF variable-width value")


def extended_value(cursor: BitCursor, width: int, escape: int) -> int:
    base = cursor.read(width)
    return base if base != escape else base + variable_value(cursor, width)


def bytes_to_bits(data: bytes) -> list[int]:
    return [
        (byte >> (7 - (index % 8))) & 1
        for byte in data
        for index in range(8)
    ]


def bits_to_bytes(bits: list[int]) -> bytes:
    if len(bits) % 8:
        raise ValueError("bit span is not byte aligned")
    output = bytearray(len(bits) // 8)
    for index, bit in enumerate(bits):
        output[index // 8] |= bit << (7 - index % 8)
    return bytes(output)


def set_bits(bits: list[int], position: int, value: int, width: int) -> None:
    for offset in range(width):
        bits[position + offset] = (value >> (width - offset - 1)) & 1


def codecdatae_positions(emdf: bytes) -> list[tuple[int, int]]:
    bits = bytes_to_bits(emdf)
    cursor = BitCursor(bits, 32)
    extended_value(cursor, 2, 3)
    extended_value(cursor, 3, 7)
    positions: list[tuple[int, int]] = []
    while True:
        payload_id = extended_value(cursor, 5, 31)
        if payload_id == 0:
            break
        sample_offset = None
        if cursor.read(1):
            sample_offset = cursor.read(11)
            if cursor.read(1):
                raise ValueError("non-zero sample-offset reserved bit")
        if cursor.read(1):
            variable_value(cursor, 11, 2)
        if cursor.read(1):
            variable_value(cursor, 2)
        codec_position = cursor.position
        codec_data_present = cursor.read(1)
        if codec_data_present:
            cursor.read(8)
        discard_unknown_payload = cursor.read(1)
        payload_frame_aligned = None
        if not discard_unknown_payload:
            if sample_offset is None:
                payload_frame_aligned = cursor.read(1)
                if payload_frame_aligned:
                    cursor.read(2)
        if sample_offset is not None or payload_frame_aligned:
            cursor.read(5)
            cursor.read(2)
        payload_size = variable_value(cursor, 8, 2)
        cursor.position += payload_size * 8
        positions.append((payload_id, codec_position))
        del group_id
    return positions


def rewrite_emdf(emdf: bytes, desired: dict[int, int]) -> bytes:
    bits = bytes_to_bits(emdf)
    positions = codecdatae_positions(emdf)
    removals: list[int] = []
    for payload_id, position in positions:
        if payload_id not in desired:
            raise ValueError(f"unexpected EMDF payload {payload_id}")
        if desired[payload_id] == 0:
            if bits[position] != 1:
                raise ValueError(f"payload {payload_id} is not codecdatae=1 in source")
            bits[position] = 0
            removals.append(position)
        else:
            bits[position] = 1
    for position in reversed(removals):
        del bits[position + 1 : position + 9]
    old_body_bytes = 0
    for bit in bits[16:32]:
        old_body_bytes = (old_body_bytes << 1) | bit
    new_body_bytes = old_body_bytes - len(removals)
    if new_body_bytes < 0:
        raise ValueError("EMDF body length underflow")
    set_bits(bits, 16, new_body_bytes, 16)
    return bits_to_bytes(bits)


def crc16(data: bytes) -> int:
    state = 0
    for byte in data:
        for shift in range(7, -1, -1):
            feedback = ((state >> 15) & 1) ^ ((byte >> shift) & 1)
            state = (state << 1) & 0xFFFF
            if feedback:
                state ^= 0x8005
    return state


def frame_size(frame: bytes) -> int:
    if frame[:2] != b"\x0b\x77":
        raise ValueError("missing E-AC-3 syncword")
    return 2 * (((frame[2] & 0x07) << 8) | frame[3]) + 2


def set_bits_in_bytes(data: bytearray, position: int, value: int, width: int) -> None:
    for offset in range(width):
        bit_position = position + offset
        byte_index = bit_position // 8
        bit_index = 7 - bit_position % 8
        mask = 1 << bit_index
        if value & (1 << (width - offset - 1)):
            data[byte_index] |= mask
        else:
            data[byte_index] &= ~mask


def copy_shifted_bits(
    destination: bytearray,
    source: bytes,
    source_start: int,
    source_end: int,
    shift_bits: int,
) -> None:
    """Copy a range left by a whole-byte amount without expanding the frame."""
    if source_start > source_end or shift_bits % 8:
        raise ValueError("codecdatae experiment requires byte-sized range shifts")
    source_byte = source_start // 8
    destination_byte = (source_start - shift_bits) // 8
    end_byte = source_end // 8
    first_remainder = source_start % 8
    if first_remainder:
        first_mask = (1 << (8 - first_remainder)) - 1
        destination[destination_byte] = (
            destination[destination_byte] & ~first_mask
        ) | (source[source_byte] & first_mask)
        source_byte += 1
        destination_byte += 1
    for index in range(source_byte, end_byte):
        destination[index - shift_bits // 8] = source[index]
    end_remainder = source_end % 8
    if end_remainder:
        tail_mask = (0xFF << (8 - end_remainder)) & 0xFF
        destination[end_byte - shift_bits // 8] = (
            destination[end_byte - shift_bits // 8] & ~tail_mask
        ) | (source[end_byte] & tail_mask)


def rewrite_frame(frame: bytes, attempt: dict[str, int], desired: dict[int, int]) -> bytes:
    start = int(attempt["frame_relative_start_bit"])
    length = int(attempt["length_bits"])
    if length % 8:
        raise ValueError("source I0 EMDF span has a non-byte length")
    frame_bits = len(frame) * 8
    old_emdf = bits_to_bytes(
        [
            (frame[bit // 8] >> (7 - bit % 8)) & 1
            for bit in range(start, start + length)
        ]
    )
    new_emdf = rewrite_emdf(old_emdf, desired)
    removed_bits = length - len(new_emdf) * 8
    if removed_bits < 0 or removed_bits % 8:
        raise ValueError("codecdatae experiment changed EMDF by a non-byte amount")
    old_end = start + length
    body_end = frame_bits - 17
    rewritten = bytearray(frame)
    new_emdf_bits = bytes_to_bits(new_emdf)
    for offset, bit in enumerate(new_emdf_bits):
        set_bits_in_bytes(rewritten, start + offset, bit, 1)
    copy_shifted_bits(rewritten, frame, old_end, body_end, removed_bits)
    for bit in range(body_end - removed_bits, body_end):
        set_bits_in_bytes(rewritten, bit, 0, 1)
    set_bits_in_bytes(rewritten, body_end, (frame[body_end // 8] >> (7 - body_end % 8)) & 1, 1)
    skip_length_position = start - 9
    set_bits_in_bytes(rewritten, skip_length_position, len(new_emdf), 9)
    output = rewritten
    output[-2:] = crc16(bytes(output[2:-2])).to_bytes(2, "big")
    return bytes(output)


def load_attempts(census_path: Path) -> dict[int, dict[str, int]]:
    census = json.loads(census_path.read_text(encoding="utf-8"))
    fixtures = census["fixtures"]
    fixture = next(
        item
        for item in fixtures
        if item["label"] == "jocforge-idx0-1024-deployed-compat"
    )
    return {int(item["syncframe"]): item for item in fixture["emdf_attempts"]}


def transform(input_path: Path, census_path: Path, output_dir: Path) -> list[dict[str, object]]:
    source = input_path.read_bytes()
    attempts = load_attempts(census_path)
    variants = {
        "A_11_1_14_1": {11: 1, 14: 1},
        "B_11_0_14_1": {11: 0, 14: 1},
        "C_11_1_14_0": {11: 1, 14: 0},
        "D_11_0_14_0": {11: 0, 14: 0},
    }
    output_dir.mkdir(parents=True, exist_ok=True)
    results: list[dict[str, object]] = []
    offset = 0
    syncframe = 0
    frames: list[tuple[int, bytes]] = []
    while offset < len(source):
        size = frame_size(source[offset:])
        frames.append((syncframe, source[offset : offset + size]))
        offset += size
        syncframe += 1
    if offset != len(source) or len(frames) != len(attempts):
        raise ValueError("source frame count does not match pinned census")
    for name, desired in variants.items():
        output = bytearray()
        for index, frame in frames:
            attempt = attempts[index]
            try:
                output.extend(rewrite_frame(frame, attempt, desired))
            except ValueError as error:
                raise ValueError(f"frame {index}: {error}") from error
        path = output_dir / f"variant-{name}.ec3"
        path.write_bytes(output)
        results.append(
            {
                "variant": name,
                "path": str(path),
                "frame_count": len(frames),
                "bytes": len(output),
                "payload_11_codecdatae": desired[11],
                "payload_14_codecdatae": desired[14],
            }
        )
    (output_dir / "experiment.json").write_text(
        json.dumps(results, indent=2) + "\n", encoding="utf-8"
    )
    return results


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("input", type=Path)
    parser.add_argument("census", type=Path)
    parser.add_argument("output_dir", type=Path)
    arguments = parser.parse_args()
    results = transform(arguments.input, arguments.census, arguments.output_dir)
    print(json.dumps(results, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
