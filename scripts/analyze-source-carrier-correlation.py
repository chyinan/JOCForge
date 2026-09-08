# pattern: Imperative Shell
"""Correlate BW64 source tracks with one captured JOCForge pre-core carrier."""

from __future__ import annotations

import argparse
import csv
import json
import math
import struct
from pathlib import Path

import numpy as np


PRE_CORE_ORDERS = {
    "ffmpeg-5.1": ("L", "R", "C", "LFE", "Ls", "Rs"),
    "jocforge-semantic-6": ("L", "R", "C", "Ls", "Rs", "LFE"),
}


def pre_core_channel_labels(order: str) -> tuple[str, ...]:
    """Return the explicit interleaved order used by the captured carrier."""
    try:
        return PRE_CORE_ORDERS[order]
    except KeyError as error:
        raise ValueError(f"unsupported pre-core channel order: {order}") from error


def locate_bw64_data(path: Path) -> tuple[int, int, int, int]:
    with path.open("rb") as handle:
        header = handle.read(12)
        if len(header) != 12 or header[0:4] not in (b"BW64", b"RF64", b"RIFF") or header[8:12] != b"WAVE":
            raise ValueError("source is not BW64/RF64/RIFF WAVE")
        data_offset = data_size = channels = bits = None
        ds64_data_size = None
        cursor = 12
        while True:
            handle.seek(cursor)
            chunk_header = handle.read(8)
            if not chunk_header:
                break
            if len(chunk_header) != 8:
                raise ValueError("source contains a truncated chunk header")
            chunk_id = chunk_header[0:4]
            declared = struct.unpack_from("<I", chunk_header, 4)[0]
            body_start = cursor + 8
            if chunk_id == b"ds64":
                handle.seek(body_start)
                body = handle.read(max(declared, 24))
                if len(body) < 24:
                    raise ValueError("source ds64 is truncated")
                ds64_data_size = struct.unpack_from("<Q", body, 8)[0]
            elif chunk_id == b"fmt ":
                handle.seek(body_start)
                body = handle.read(declared)
                if len(body) < 16:
                    raise ValueError("source fmt is truncated")
                format_tag, channels, _, _, block_align, bits = struct.unpack_from("<HHIIHH", body)
                if format_tag != 1 or bits != 24 or block_align != channels * 3:
                    raise ValueError("source must be signed PCM24")
            body_size = ds64_data_size if chunk_id == b"data" and declared == 0xFFFFFFFF else declared
            if body_size is None:
                raise ValueError(f"source chunk {chunk_id!r} has no resolved size")
            if chunk_id == b"data":
                data_offset, data_size = body_start, body_size
            cursor = body_start + body_size + body_size % 2
        if None in (data_offset, data_size, channels, bits):
            raise ValueError("source is missing data/fmt")
        return int(data_offset), int(data_size), int(channels), int(bits)


def decode_int24(raw: bytes, channels: int) -> np.ndarray:
    values = np.frombuffer(raw, dtype=np.uint8).reshape(-1, channels, 3)
    unsigned = (
        values[:, :, 0].astype(np.int32)
        | (values[:, :, 1].astype(np.int32) << 8)
        | (values[:, :, 2].astype(np.int32) << 16)
    )
    signed = unsigned - ((unsigned >= 0x800000) * 0x1000000).astype(np.int32)
    return signed.astype(np.float64) / 8_388_608.0


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("source", type=Path)
    parser.add_argument("pre_core", type=Path)
    parser.add_argument("root", type=Path)
    parser.add_argument(
        "--pre-core-order",
        choices=tuple(PRE_CORE_ORDERS),
        default="ffmpeg-5.1",
        help="interleaved order used when the pre-core carrier was captured",
    )
    args = parser.parse_args()
    pre_core_labels = pre_core_channel_labels(args.pre_core_order)
    data_offset, data_size, channels, _ = locate_bw64_data(args.source)
    if channels != 38 or data_size % (channels * 3):
        raise ValueError("verified Hanabi source must contain 38 PCM tracks")
    source_frames = data_size // (channels * 3)
    pre = np.memmap(args.pre_core, dtype="<f4", mode="r")
    if pre.size % 6:
        raise ValueError("pre-core capture is not a six-channel float32 stream")
    frame_count = min(source_frames, pre.size // 6)
    pre = pre.reshape(-1, 6)[:frame_count].astype(np.float64)
    sums = np.zeros(channels, dtype=np.float64)
    squares = np.zeros(channels, dtype=np.float64)
    cross = np.zeros((6, channels), dtype=np.float64)
    source_peaks = np.zeros(channels, dtype=np.float64)
    pre_sum = np.sum(pre, axis=0)
    pre_square = np.sum(pre * pre, axis=0)
    block_frames = 65_536
    with args.source.open("rb") as handle:
        handle.seek(data_offset)
        processed = 0
        while processed < frame_count:
            count = min(block_frames, frame_count - processed)
            raw = handle.read(count * channels * 3)
            decoded = decode_int24(raw, channels)
            if decoded.shape != (count, channels):
                raise ValueError("source data ended before the declared frame count")
            sums += np.sum(decoded, axis=0)
            squares += np.sum(decoded * decoded, axis=0)
            cross += pre[processed : processed + count].T @ decoded
            source_peaks = np.maximum(source_peaks, np.max(np.abs(decoded), axis=0))
            processed += count
    pre_centered = pre - pre_sum / frame_count
    pre_centered_square = np.sum(pre_centered * pre_centered, axis=0)
    rows: list[dict[str, float | int]] = []
    for pre_channel in range(6):
        for track in range(channels):
            mean = sums[track] / frame_count
            variance = max(squares[track] - frame_count * mean * mean, 0.0)
            centered_cross = cross[pre_channel, track] - mean * pre_sum[pre_channel]
            centered_denominator = math.sqrt(variance * pre_centered_square[pre_channel])
            rms = math.sqrt(squares[track] / frame_count)
            rows.append(
                {
                    "pre_core_channel": pre_channel,
                    "track_index": track + 1,
                    "rms": rms,
                    "rms_db": 20.0 * math.log10(max(rms, 1.0e-30)),
                    "peak": float(source_peaks[track]),
                    "normalized_dot": float(cross[pre_channel, track] / math.sqrt(max(squares[track] * pre_square[pre_channel], 1.0e-30))),
                    "zero_mean_correlation": float(centered_cross / centered_denominator) if centered_denominator > 0.0 else 0.0,
                }
            )
    rows.sort(key=lambda row: (int(row["pre_core_channel"]), -abs(float(row["zero_mean_correlation"]))),)
    report = {
        "schema": "jocforge.flat7x.source-precore-correlation.v1",
        "source_frames": source_frames,
        "compared_frames": frame_count,
        "source_channels": channels,
        "pre_core_order": args.pre_core_order,
        "pre_core_channels": list(pre_core_labels),
        "tracks_sorted_by_pre_core_channel_and_abs_zero_mean_correlation": rows,
    }
    args.root.mkdir(parents=True, exist_ok=True)
    (args.root / "source-track-precore-correlation.json").write_text(json.dumps(report, indent=2), encoding="utf-8")
    with (args.root / "source-track-precore-correlation.csv").open("w", newline="", encoding="utf-8") as handle:
        writer = csv.DictWriter(handle, fieldnames=rows[0].keys())
        writer.writeheader()
        writer.writerows(rows)
    print("compared_frames", frame_count)
    for pre_channel in range(6):
        selected = [row for row in rows if int(row["pre_core_channel"]) == pre_channel][:5]
        print("pre_core", pre_channel, [(row["track_index"], round(float(row["zero_mean_correlation"]), 6)) for row in selected])
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
