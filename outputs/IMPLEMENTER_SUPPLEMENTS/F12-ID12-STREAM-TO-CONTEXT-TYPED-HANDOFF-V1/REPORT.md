# F12 ID12 stream-to-context typed handoff

## Result

The native selected path now statically binds the per-stream ID12 dword at `stream_table[index] + 0x9c` to a nested frame context field. A pointer-type correction matters: `FUN_140317600` receives `param_2` as `longlong*`, so `param_2 + 0x0c` is `stream + 0x60`; the later `[RCX+0x3c]` read is therefore `stream + 0x9c`, not `stream + 0x0c`.

The value is copied as a 32-bit dword into nested-context `+0x34`, then its low word is written to context `+0x19140`. Two frame serializer consumers read `+0x19140` while receiving a frame record. This closes a typed stream-to-context handoff for the selected static path, not the final BSI U5 rule: the field-to-wire selector join remains open for Agent 5, and no runtime generation token was captured.

## Hash-bound source evidence

Source package: `multi-adm-dialnorm-cross-input`, MANIFEST SHA-256 `59dd26bea5a4bed89544e06d62d365cc2f9d58b4241c3c1f1b060d08d730faa7`. Its normal and optimized checkers both passed. The package contains research/decompilation payloads; none are copied into this supplement.

The source package binds `dee_ddp_encoder.exe` SHA-256 `0cf23520799f52323b0994e5fcc16924da7b027a5b83a0cff3b4f06792c18532` and `dee_audio_filter_ddp_atmos.dll` SHA-256 `3d66bcec36031fd48e6565d15f05fea656642377ca4f8c98cdce1cce8b7e95d2`.

### Static transfer

1. `FUN_140315db0` dispatches ID12 setter and frame-output methods on the same stream receiver. The stream pointer table and common-index processing loops use distinct count fields; their common range is statically bounded, but the runtime generation token is not captured.
2. The API field is a 32-bit dword at `stream_table[index] + 0x9c`, with the source package's declared legal range `[1,31]`.
3. In the selected frame-output path, `FUN_140317600` passes `stream+0x60` as the source base to `FUN_1403471f0`. That helper reads `[RCX+0x3c]`, which resolves to `stream+0x9c`, then writes the dword to `nested_context+0x34`.
4. `FUN_140348000` copies the low word from context `+0x34` to context `+0x19140`; an alternate branch performs the corresponding copy. `FUN_140343530` and `FUN_1403448d0` later read that context field while consuming the current frame record.

## Limits and next edge

- **CLOSED_STATIC:** typed ID12 stream field → nested context `+0x34` → context `+0x19140` → frame serializer input.
- **UNBOUND:** context `+0x19140` to the final U5 selector/BSI write; the native generation token is not captured.
- **NOT_RUN:** no DEE run, PCM/EC3 payload read, or dynamic output observation.

The next minimum closure is Agent 5's same-image selector/write path: determine whether the exact context field `+0x19140` participates in the row at `owner+0x1bae8` and is emitted as U5 in the selected 1024 kbps frame. Do not infer that relation from similar numeric values or separate callers. No key, HMAC, license, source code, decompilation, binary, PCM, or output payload is included here.
