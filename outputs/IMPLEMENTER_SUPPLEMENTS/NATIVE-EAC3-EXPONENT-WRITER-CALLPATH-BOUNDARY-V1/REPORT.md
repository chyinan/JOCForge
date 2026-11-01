# Native E-AC-3 exponent writer call-path boundary

## Result

The current Rust selected-frame source statically emits an active fullband lane as U6 bandwidth code (first pass over lanes), then U4 base exponent, 72 U7 grouped exponent codes, and U2 gain range. The LFE lane emits U4 base plus two U7 groups before BA-update syntax. The attempt-10 lane/block/strategy is not bound to this plan.

The hash-bound Ghidra 12.2 path in `dee_audio_filter_ddp_atmos.dll` reaches F32 coefficient/exponent quantization and per-item output packing, but it does not expose the corresponding U4 base/U7 grouped frame-syntax writer in the bounded path. The native wire writer, attempt-10 generation/profile join, and causal link to the exponent `-1` diagnostic remain `UNBOUND`.

The reused parser-side single-group vector demonstrates that a 4-bit initial exponent plus one 7-bit group can underflow to `-1`; it is not a full syncframe and does not model the Rust 72-group lane, its next-U7 field, or attempt-10's actual lane/cursor. This is an audit hypothesis only, not evidence that Rust is wrong.

## Evidence source

Source package: `native-eac3-writer-calllineage-wash-v1`, MANIFEST SHA-256 `9cae34b4d7a0cd32908207de1a9cfdc87f433581ecb9319f404b33e93fc8951e`. Its normal and optimized checkers both passed; no output bytes were included or read.

Target native image: `dee_audio_filter_ddp_atmos.dll`, SHA-256 `3d66bcec36031fd48e6565d15f05fea656642377ca4f8c98cdce1cce8b7e95d2`, verified against the prior sealed image identity. Ghidra traced the bounded path through `FUN_1801febe0 → FUN_1801bcf40 → FUN_1801f0bd0 → FUN_1801d7020 → FUN_1801d6df0/FUN_1801d5ed0`. The observed F32 numeric scale and the literal item-count argument 6 are not U4/U7 wire syntax.

## Next minimum comparison

Find the hash-bound native U4/U7 frame-syntax emitter, bind its strategy/group-count inputs and bit cursor across two adjacent U7 groups to the selected 1024 kbps AU 0 lane/block, then compare the same fields against the Rust plan. Do not infer from the parser's one-group vector. No private HMAC key, license, PCM, or EC3 payload was read.
