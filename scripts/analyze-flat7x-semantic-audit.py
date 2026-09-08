# pattern: Imperative Shell
"""Measure one-hot Flat-7.X render identity from audit WAV files."""

from __future__ import annotations

import argparse
import csv
import json
import math
from pathlib import Path

import numpy as np


INPUTS = ("L", "R", "C", "Ls", "Rs", "Lrs", "Rrs", "LFE")
OUTPUTS = ("FL", "FR", "FC", "LFE", "Lb", "Rb", "Ls", "Rs")
EXPECTED_OUTPUTS = {
    "L": "FL",
    "R": "FR",
    "C": "FC",
    "Ls": "Ls",
    "Rs": "Rs",
    "Lrs": "Lb",
    "Rrs": "Rb",
    "LFE": "LFE",
}
ALLOWED_OUTPUTS = {
    **{label: {output} for label, output in EXPECTED_OUTPUTS.items()},
    "Lrs": {"Lb", "Rb"},
    "Rrs": {"Lb", "Rb"},
}
MAX_UNINTENDED_RMS_RATIO = 0.01
MIN_IDENTITY_RMS = 1.0e-6


def read_float_wav(path: Path) -> np.ndarray:
    blob = path.read_bytes()
    if len(blob) < 12 or blob[0:4] != b"RIFF" or blob[8:12] != b"WAVE":
        raise ValueError(f"{path} is not a RIFF/WAVE file")
    fmt: bytes | None = None
    pcm: bytes | None = None
    cursor = 12
    while cursor + 8 <= len(blob):
        chunk_id = blob[cursor : cursor + 4]
        size = int.from_bytes(blob[cursor + 4 : cursor + 8], "little")
        start = cursor + 8
        end = start + size
        if end > len(blob):
            raise ValueError(f"{path} contains an out-of-bounds {chunk_id!r} chunk")
        if chunk_id == b"fmt ":
            fmt = blob[start:end]
        elif chunk_id == b"data":
            pcm = blob[start:end]
        cursor = end + size % 2
    if fmt is None or pcm is None or len(fmt) < 16:
        raise ValueError(f"{path} is missing fmt/data")
    format_tag = int.from_bytes(fmt[0:2], "little")
    channels = int.from_bytes(fmt[2:4], "little")
    rate = int.from_bytes(fmt[4:8], "little")
    block_align = int.from_bytes(fmt[12:14], "little")
    width = int.from_bytes(fmt[14:16], "little")
    is_float = format_tag == 3 or (format_tag == 0xFFFE and len(fmt) >= 40 and fmt[24:26] == b"\x03\x00")
    if channels != len(OUTPUTS) or width != 32 or rate != 48_000 or not is_float:
        raise ValueError(f"{path} is not an 8-channel 48 kHz float32 WAV")
    if block_align != channels * 4 or len(pcm) % block_align:
        raise ValueError(f"{path} has invalid float32 PCM dimensions")
    return np.frombuffer(pcm, dtype="<f4").reshape(-1, channels).copy()


def channel_metrics(samples: np.ndarray) -> list[dict[str, float | bool]]:
    if samples.ndim != 2 or samples.shape[1] != len(OUTPUTS):
        raise ValueError("decoded WAV shape does not match the canonical 7.1 output")
    if not np.isfinite(samples).all():
        raise ValueError("decoded WAV contains non-finite samples")
    values: list[dict[str, float | bool]] = []
    for index, label in enumerate(OUTPUTS):
        channel = samples[:, index].astype(np.float64)
        rms = float(np.sqrt(np.mean(channel * channel))) if len(channel) else 0.0
        peak = float(np.max(np.abs(channel))) if len(channel) else 0.0
        values.append(
            {
                "channel": label,
                "rms": rms,
                "rms_db": float(20.0 * math.log10(max(rms, 1.0e-30))),
                "peak": peak,
                "finite": True,
            }
        )
    return values


def validate_identity_report(rows: list[dict[str, object]], full_finite: bool) -> list[str]:
    """Return actionable failures for the permanent one-hot semantic gate."""
    failures: list[str] = []
    by_input = {str(row.get("input")): row for row in rows}
    for input_label, expected_output in EXPECTED_OUTPUTS.items():
        row = by_input.get(input_label)
        if row is None:
            failures.append(f"{input_label} row is missing")
            continue
        if row.get("dominant_output") != expected_output:
            failures.append(
                f"{input_label} expected {expected_output}, got {row.get('dominant_output')}"
            )
        if not bool(row.get("finite")):
            failures.append(f"{input_label} render is non-finite")
        if float(row.get("max_rms", 0.0)) <= MIN_IDENTITY_RMS:
            failures.append(f"{input_label} render is silent")
        normalized = row.get("normalized_rms")
        if not isinstance(normalized, list) or len(normalized) != len(OUTPUTS):
            failures.append(f"{input_label} normalized RMS vector is invalid")
            continue
        for output_index, output_label in enumerate(OUTPUTS):
            if output_label not in ALLOWED_OUTPUTS[input_label] and float(normalized[output_index]) > MAX_UNINTENDED_RMS_RATIO:
                failures.append(
                    f"{input_label} leaks into {output_label}: ratio {float(normalized[output_index]):.6g}"
                )
    if not full_finite:
        failures.append("full render is non-finite")
    return failures


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("root", type=Path)
    parser.add_argument("--prefix", default="identity-")
    parser.add_argument("--output-stem", default="channel-identity-matrix")
    args = parser.parse_args()
    root = args.root
    rows: list[dict[str, object]] = []
    matrix: list[list[float]] = []
    for input_label in INPUTS:
        samples = read_float_wav(root / f"{args.prefix}{input_label}.wav")
        metrics = channel_metrics(samples)
        rms = np.array([float(item["rms"]) for item in metrics], dtype=np.float64)
        maximum = float(np.max(rms)) if len(rms) else 0.0
        normalized = (rms / maximum).tolist() if maximum > 0.0 else [0.0] * len(OUTPUTS)
        dominant = OUTPUTS[int(np.argmax(rms))] if maximum > 0.0 else "SILENT"
        rows.append(
            {
                "input": input_label,
                "dominant_output": dominant,
                "max_rms": maximum,
                "finite": all(bool(item["finite"]) for item in metrics),
                "normalized_rms": normalized,
                "channels": metrics,
            }
        )
        matrix.append(normalized)
    full_samples = read_float_wav(root / f"{args.prefix}full.wav")
    report = {
        "schema": "jocforge.flat7x.audio-semantic-identity.v1",
        "input_order": list(INPUTS),
        "output_order": list(OUTPUTS),
        "one_hot": rows,
        "normalized_rms_matrix": matrix,
        "center_only_dominant_output": rows[2]["dominant_output"],
        "lfe_only_dominant_output": rows[7]["dominant_output"],
        "full_render": {
            "frames": int(full_samples.shape[0]),
            "finite": bool(np.isfinite(full_samples).all()),
            "channels": channel_metrics(full_samples),
        },
    }
    gate_failures = validate_identity_report(rows, bool(report["full_render"]["finite"]))
    report["semantic_gate"] = {
        "passed": not gate_failures,
        "max_unintended_rms_ratio": MAX_UNINTENDED_RMS_RATIO,
        "minimum_identity_rms": MIN_IDENTITY_RMS,
        "failures": gate_failures,
    }
    root.mkdir(parents=True, exist_ok=True)
    (root / f"{args.output_stem}.json").write_text(
        json.dumps(report, indent=2), encoding="utf-8"
    )
    with (root / f"{args.output_stem}.csv").open("w", newline="", encoding="utf-8") as handle:
        writer = csv.writer(handle)
        writer.writerow(["input", *OUTPUTS, "dominant_output"])
        for row, values in zip(rows, matrix):
            writer.writerow([row["input"], *[f"{value:.9f}" for value in values], row["dominant_output"]])
    for row in rows:
        print(row["input"], "->", row["dominant_output"], "max_rms", f"{float(row['max_rms']):.9g}")
    print("center_only_dominant_output", report["center_only_dominant_output"])
    print("lfe_only_dominant_output", report["lfe_only_dominant_output"])
    print("full_render_frames", report["full_render"]["frames"])
    if gate_failures:
        print("semantic_identity_gate FAIL")
        for failure in gate_failures:
            print(" -", failure)
        return 1
    print("semantic_identity_gate PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
