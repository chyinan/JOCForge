# silero-vad-pure

A **pure-Rust inference engine** for [silero-vad](https://github.com/snakers4/silero-vad) (v6.2) — no ONNX Runtime, no PyTorch, no inference frameworks at all. The full forward pass (STFT → 4× Conv1d+ReLU → LSTM → sigmoid head) is hand-implemented in ~1400 lines of Rust, with NEON kernels on aarch64 and Accelerate/`matrixmultiply` sgemm backends.

![benchmark](assets/benchmark.png)

**~3–6× faster than the official runtimes**, single-threaded, same 32 ms chunks (Apple M5 Pro; reproduce with `cargo run --release --example bench` and `.venv/bin/python tools/bench_compare.py`). Validated bit-faithful against the onnxruntime reference: probability max error ≤ 1.2e-7 over synthetic streams and 5286 chunks of real speech at both sample rates.

## Features

- Weights **embedded at compile time** (~2.2 MB, both 8kHz & 16kHz sets) — single binary, zero runtime assets
- Streaming stateful API: 512-sample (16kHz) / 256-sample (8kHz) chunks
- NEON-optimized: direct STFT/conv kernels (no im2col), vectorized sigmoid/tanh, fused LSTM gate update
- Portable fallback: `--no-default-features` → pure-Rust `matrixmultiply` backend (still ~1191× real-time at 16kHz)

## Usage

```rust
use silero_vad_pure::{SampleRate, SileroVad};

let mut vad = SileroVad::new(SampleRate::Hz16000)?;
for chunk in audio.chunks_exact(vad.chunk_size()) {
    let speech_prob: f32 = vad.process(chunk)?; // LSTM state carried over
}
vad.reset(); // new stream
```

## Build & test

```bash
cargo test                              # golden tests vs onnxruntime reference
cargo run --release --example bench     # per-chunk latency
```

Details (per-stage timings, methodology caveats, weight export tooling) live in `BENCHMARKS.md`, `PLAN.md`, and `tools/`.

## License & acknowledgments

Model weights and architecture from [snakers4/silero-vad](https://github.com/snakers4/silero-vad) (MIT). Inference re-implemented from scratch in Rust.

## JOCForge integration

This source matches crates.io silero-vad-pure 0.1.1 with two local API changes in src/vad.rs. The input accepts upstream Silero streaming context in addition to the 32 ms frame, and restore_state restores both recurrent vectors previously returned by state(). Weights and model kernels come from the published crate.
