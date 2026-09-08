# pattern: Imperative Shell
"""Build deterministic Flat-7.X one-hot ADM fixtures for semantic forensics."""

from __future__ import annotations

import argparse
import math
import re
import struct
from pathlib import Path


SAMPLE_RATE = 48_000
IDENTITIES = ("L", "R", "C", "Ls", "Rs", "Lrs", "Rrs")
FREQUENCIES_HZ = (401.0, 503.0, 607.0, 719.0, 829.0, 941.0, 1061.0)
IDENTITY_POSITIONS = (
    (-1.0, -1.0, 0.0),  # L -> renderer FL
    (1.0, -1.0, 0.0),  # R -> renderer FR
    (0.0, -1.0, 0.0),  # C -> renderer FC
    (-1.0, 0.0, 0.0),  # Ls -> renderer Ls
    (1.0, 0.0, 0.0),  # Rs -> renderer Rs
    (-1.0, 1.0, 0.0),  # Lrs -> renderer Lb
    (1.0, 1.0, 0.0),  # Rrs -> renderer Rb
)
SHUFFLED_POSITIONS = (
    IDENTITY_POSITIONS[2],  # source slot 0 is spatially C
    IDENTITY_POSITIONS[1],
    IDENTITY_POSITIONS[0],  # source slot 2 is spatially L
    IDENTITY_POSITIONS[3],
    IDENTITY_POSITIONS[4],
    IDENTITY_POSITIONS[5],
    IDENTITY_POSITIONS[6],
)


def chunk_table(blob: bytes) -> list[tuple[bytes, bytes, int]]:
    if len(blob) < 12 or blob[0:4] not in (b"BW64", b"RF64", b"RIFF") or blob[8:12] != b"WAVE":
        raise ValueError("input is not a supported BW64/RF64/RIFF WAVE")
    chunks: list[tuple[bytes, bytes, int]] = []
    cursor = 12
    data_size: int | None = None
    while cursor + 8 <= len(blob):
        chunk_id = blob[cursor : cursor + 4]
        declared = struct.unpack_from("<I", blob, cursor + 4)[0]
        body_start = cursor + 8
        if chunk_id == b"ds64" and declared >= 16:
            data_size = struct.unpack_from("<Q", blob, body_start + 8)[0]
        body_size = data_size if chunk_id == b"data" and declared == 0xFFFFFFFF else declared
        if body_size is None or body_start + body_size > len(blob):
            raise ValueError(f"invalid chunk bounds for {chunk_id!r}")
        chunks.append((chunk_id, blob[body_start : body_start + body_size], declared))
        cursor = body_start + body_size + body_size % 2
    if cursor != len(blob):
        raise ValueError("input contains an incomplete trailing chunk")
    return chunks


def pcm_format(chunks: list[tuple[bytes, bytes, int]]) -> tuple[int, int, int]:
    fmt = next((body for chunk_id, body, _ in chunks if chunk_id == b"fmt "), None)
    data = next((body for chunk_id, body, _ in chunks if chunk_id == b"data"), None)
    if fmt is None or data is None or len(fmt) < 16:
        raise ValueError("fixture is missing fmt/data")
    format_tag, channels, sample_rate, _, block_align, bits = struct.unpack_from("<HHIIHH", fmt)
    if format_tag != 1 or sample_rate != SAMPLE_RATE or bits != 24:
        raise ValueError("fixture must be 48 kHz signed PCM24")
    if block_align != channels * 3 or channels == 0 or len(data) % block_align:
        raise ValueError("fixture has invalid PCM dimensions")
    return channels, sample_rate, len(data) // block_align


def replace_object_positions(xml: bytes, positions: tuple[tuple[float, float, float], ...]) -> bytes:
    text = xml.decode("utf-8")
    for axis_index, axis in enumerate(("X", "Y", "Z")):
        pattern = re.compile(rf'(<position coordinate="{axis}">)[^<]*(</position>)')
        matches = list(pattern.finditer(text))
        if len(matches) != len(positions):
            raise ValueError(f"expected {len(positions)} {axis} positions, found {len(matches)}")
        replacement_values = [f"{position[axis_index]:.9f}" for position in positions]
        iterator = iter(replacement_values)
        text = pattern.sub(lambda match: f"{match.group(1)}{next(iterator)}{match.group(2)}", text)
    return text.encode("utf-8")


def int24(value: float) -> bytes:
    scaled = max(-8_388_608, min(8_388_607, round(value * 8_388_607.0)))
    return int(scaled).to_bytes(4, "little", signed=True)[:3]


def build_pcm(sample_count: int, active: int | None) -> bytes:
    output = bytearray(sample_count * 7 * 3)
    for sample in range(sample_count):
        time = sample / SAMPLE_RATE
        for channel, frequency in enumerate(FREQUENCIES_HZ):
            value = 0.16 * math.sin(2.0 * math.pi * frequency * time)
            if active is not None and channel != active:
                value = 0.0
            offset = (sample * 7 + channel) * 3
            output[offset : offset + 3] = int24(value)
    return bytes(output)


def build_lfe_pcm(sample_count: int) -> bytes:
    channels = 5  # three DirectSpeakers channels, then two Objects channels
    output = bytearray(sample_count * channels * 3)
    for sample in range(sample_count):
        time = sample / SAMPLE_RATE
        values = (0.0, 0.0, 0.20 * math.sin(2.0 * math.pi * 73.0 * time), 0.0, 0.0)
        for channel, value in enumerate(values):
            offset = (sample * channels + channel) * 3
            output[offset : offset + 3] = int24(value)
    return bytes(output)


def rebuild_with_header(header: bytes, chunks: list[tuple[bytes, bytes, int]], replacements: dict[bytes, bytes]) -> bytes:
    bodies = [(chunk_id, replacements.get(chunk_id, body), declared) for chunk_id, body, declared in chunks]
    total_size = 12 + sum(8 + len(body) + len(body) % 2 for _, body, _ in bodies)
    output = bytearray(header)
    for chunk_id, body, declared in bodies:
        output.extend(chunk_id)
        output.extend(struct.pack("<I", 0xFFFFFFFF if chunk_id == b"data" and declared == 0xFFFFFFFF else len(body)))
        output.extend(body)
        if len(body) % 2:
            output.append(0)
    if len(output) != total_size:
        raise AssertionError("rebuilt fixture size accounting mismatch")
    ds64_offset = output.find(b"ds64")
    if ds64_offset < 0:
        raise ValueError("BW64 fixture is missing ds64")
    struct.pack_into("<Q", output, ds64_offset + 8, total_size - 8)
    return bytes(output)


def build_fixture(base: Path, output: Path, mode: str) -> None:
    blob = base.read_bytes()
    chunks = chunk_table(blob)
    channels, _, sample_count = pcm_format(chunks)
    if mode == "lfe":
        if channels != 5:
            raise ValueError("LFE fixture must contain three DirectSpeakers plus two Objects tracks")
        pcm = build_lfe_pcm(sample_count)
        replacements = {b"data": pcm}
    else:
        if channels != 7:
            raise ValueError("Flat-7.X fixture must contain seven object tracks")
        shuffled = mode.startswith("shuffle")
        active = None if mode == "full" else (int(mode.removeprefix("shuffle")) if shuffled else IDENTITIES.index(mode))
        pcm = build_pcm(sample_count, active)
        xml = next(body for chunk_id, body, _ in chunks if chunk_id == b"axml")
        replacements = {
            b"data": pcm,
            b"axml": replace_object_positions(xml, SHUFFLED_POSITIONS if shuffled else IDENTITY_POSITIONS),
        }
    rebuilt = rebuild_with_header(blob[:12], chunks, replacements)
    if len(rebuilt) != len(blob) and b"axml" not in replacements:
        raise ValueError("unexpected fixture size change")
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_bytes(rebuilt)
    if len(rebuilt) == 0 or not math.isfinite(float(len(rebuilt))):
        raise ValueError("fixture write produced an invalid result")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("base", type=Path)
    parser.add_argument("output", type=Path)
    parser.add_argument("mode", choices=("full", *IDENTITIES, *(f"shuffle{index}" for index in range(7)), "lfe"))
    args = parser.parse_args()
    build_fixture(args.base, args.output, args.mode)
    print(f"wrote {args.mode} fixture: {args.output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
