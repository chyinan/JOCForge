"""Local diagnostic for object loss. Pass a built JOCForge CLI and OpenJOC."""

import argparse
from pathlib import Path
import struct
import subprocess
import tempfile

import numpy as np
from scipy.io import wavfile


def chunk(tag, payload):
    return tag + struct.pack("<I", len(payload)) + payload + bytes(len(payload) % 2)


def write_adm(path, pcm):
    fixtures = Path(__file__).resolve().parents[1] / "crates/jocforge-domain/tests/fixtures/selected_adm_graph"
    rate, channels = 48000, pcm.shape[1]
    fmt = struct.pack("<HHIIHH", 1, channels, rate, rate * channels * 3, channels * 3, 24)
    body = b"WAVE" + chunk(b"fmt ", fmt)
    for tag, name in [(b"axml", "source_metadata.xml"), (b"chna", "source_track_associations.bin"),
                      (b"dbmd", "source_metadata.dbmd")]:
        body += chunk(tag, (fixtures / name).read_bytes())
    body += chunk(b"data", pcm.tobytes())
    path.write_bytes(b"RIFF" + struct.pack("<I", len(body)) + body)


def marker_pcm():
    rate, channels, frame = 48000, 38, 1536
    sample = np.arange(10 * frame)
    fade = np.minimum(1, np.minimum(sample, sample[::-1]) / 240)
    tone = (0.125 * 8388607 * np.sin(2 * np.pi * 997 * sample / rate) * fade).astype("<i4")
    packed = tone.view("u1").reshape(-1, 4)[:, :3]
    pcm = np.zeros((56 * frame, channels, 3), dtype="u1")
    for row, start in [(0, 8 * frame), (10, 32 * frame)]:  # front left, then ADM object
        pcm[start:start + len(sample), row] = packed
    return pcm


def continuous_pcm():
    # A continuous front-left tone with changing competition for the I0 frame budget.
    frame = 1536
    samples = np.zeros((96 * frame, 38))
    samples[:, 0] = 0.1 * np.sin(2 * np.pi * 997 * np.arange(len(samples)) / 48000)
    rng = np.random.default_rng(731)
    for index, (count, level) in enumerate([(1, .08), (2, .08), (3, .08), (4, .08),
                                            (4, .012), (5, .003), (4, .001), (3, .003)]):
        segment = slice(index * 12 * frame, (index + 1) * 12 * frame)
        for channel in [1, 2, 4, 5, 3][:count]:
            samples[segment, channel] = level * rng.uniform(-1, 1, 12 * frame)
    return (samples * 8388607).astype("<i4").view("u1").reshape(-1, 38, 4)[:, :, :3]


def check_fidelity(encoder, openjoc, work):
    """Compare the decoded samples with independent waveforms on known input tracks."""
    from scipy.signal import butter, correlate, sosfilt

    frame = 1536
    count = 96 * frame
    rng = np.random.default_rng(137)
    pcm = np.zeros((count, 38))
    fullband = [0, 1, 2, 4, 5, 6, 7]
    lowpass = butter(6, 8000, fs=48000, output="sos")
    for channel in fullband:
        pcm[:48 * frame, channel] = sosfilt(lowpass, rng.normal(0, .04, 48 * frame))
    pcm[48 * frame:, 10] = sosfilt(lowpass, rng.normal(0, .04, 48 * frame))
    source = work / "wideband.wav"
    write_adm(source, (pcm * 8388607).astype("<i4").view("u1").reshape(count, 38, 4)[:, :, :3])
    attempt = work / "attempt"
    attempt.mkdir()
    stream = attempt / "wideband.candidate.ec3"
    run([encoder, "encode", source, stream, "--joc-layout", "flat7x"], work)
    decoded = subprocess.check_output(["ffmpeg", "-v", "error", "-drc_scale", "0", "-i", stream,
                                       "-f", "f32le", "pipe:1"])
    base = np.frombuffer(decoded, "<f4").reshape(count, 8)
    # FFmpeg outputs backs before sides. Every decoded lane has the MDCT overlap delay.
    for host, output in zip(fullband, [0, 1, 2, 6, 7, 4, 5]):
        reference = pcm[12000:60000, host]
        actual = base[12256:60256, output]
        gain = np.dot(actual, reference) / np.dot(reference, reference)
        rho = np.corrcoef(actual, reference)[0, 1]
        if not (0.93 <= gain <= 1.07 and rho > .97):
            raise SystemExit(f"carrier {host}: gain={gain:.4f}, aligned correlation={rho:.4f}")

    rendered = work / "objects.wav"
    run([openjoc, "render-joc", stream, "--layout", "7.1", "--diagnostic-contribution",
         "reconstruction-only", "--validation-profile", "observed-vendor-compat",
         "--output", rendered, "--no-progress"], work)
    rate, objects = wavfile.read(rendered)
    if rate != 48000 or objects.shape[1] != 8:
        raise SystemExit(f"unexpected object render: {rate} Hz, {objects.shape}")
    # The fixture's first object is on the left near the rear wall.
    actual = objects[90000:130000, 4]
    reference = pcm[89000:131000, 10]
    start = np.argmax(np.abs(correlate(reference, actual, "valid", method="fft")))
    delay = 1000 - start
    rho = np.corrcoef(actual, reference[start:start + len(actual)])[0, 1]
    print(f"seven base channels aligned; isolated object delay={delay}, correlation={rho:.4f}")
    if delay != 288 or not rho > .90:
        raise SystemExit(1)


def check_core_continuity(stream):
    frames = np.frombuffer(stream.read_bytes(), dtype="u1").reshape(96, 6656)
    decoded = subprocess.run(
        ["ffmpeg", "-v", "error", "-drc_scale", "0", "-f", "ac3", "-i", "pipe:0",
         "-af", "pan=mono|c0=c0", "-f", "f32le", "pipe:1"],
        input=frames[:, :2560].tobytes(), stdout=subprocess.PIPE, check=True,
    )
    front_left = np.frombuffer(decoded.stdout, "<f4").reshape(96, 1536)
    silent = np.flatnonzero(np.max(np.abs(front_left), axis=1) == 0)
    print(f"silent front-left frames: {len(silent)}/96")
    if len(silent):
        raise SystemExit(1)


def run(command, directory):
    result = subprocess.run(command, cwd=directory, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    if result.returncode:
        raise SystemExit(result.stdout.decode(errors="replace")[-2000:])


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("encoder", type=Path)
    parser.add_argument("openjoc", type=Path)
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--fidelity", action="store_true", help="check carrier gain, timing and object reconstruction")
    mode.add_argument("--continuity", action="store_true", help="check Flat7x I0 frame dropouts with FFmpeg")
    mode.add_argument("--baseline", action="store_true", help="test the selected 5.1 encode")
    args = parser.parse_args()
    with tempfile.TemporaryDirectory() as tmp:
        work = Path(tmp)
        if args.fidelity:
            check_fidelity(args.encoder.resolve(), args.openjoc.resolve(), work)
            return
        source = work / "markers.wav"
        write_adm(source, continuous_pcm() if args.continuity else marker_pcm())
        attempt = work / "attempt"
        attempt.mkdir()
        stream = attempt / ("markers.ec3" if args.baseline else "markers.candidate.ec3")
        layout = [] if args.baseline else ["--joc-layout", "flat7x"]
        run([args.encoder.resolve(), "encode", source, stream, *layout], work)
        if args.continuity:
            check_core_continuity(stream)
            return
        rendered = work / "render.wav"
        run([args.openjoc.resolve(), "render-joc", stream, "--layout", "7.1", "--output", rendered,
             "--validation-profile", "observed-vendor-compat", "--no-progress"], work)
        rate, audio = wavfile.read(rendered)
        if rate != 48000 or audio.shape[1] != 8 or len(audio) < 56 * 1536:
            raise SystemExit(f"Unexpected render: {rate} Hz, shape {audio.shape}")
        split = round(0.82 * rate)
        bed = np.max(np.abs(audio[:split]))
        obj = np.max(np.abs(audio[split:56 * 1536]))
        print(f"front-left marker peak={bed:.8f}  object marker peak={obj:.8f}")
        if not np.isfinite(audio).all() or bed == 0 or obj < 0.1 * bed:
            raise SystemExit(1)


if __name__ == "__main__":
    main()
