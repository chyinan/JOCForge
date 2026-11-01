# F13C Atmos callback registration boundary

## Static handoff

The v8 trace resolved `FUN_1403c3980` as an `AtmosMasterFileStorage::Factory` child method. This v9 continuation follows `FUN_1403f5270`: it allocates a `0x6380` object, calls `FUN_1403f1dd0` to initialize a storage callback wrapper, then forwards the post-call RAX value through `FUN_1403f1a50` into `FUN_1403a6960` with a `0x18` holder. The helper reads the holder/owner field at `+0x8`, tests it, and branches at `0x1403a6984` to `0x1403a69e3` or fallthrough `0x1403a6986`.

The v6/v7/v8 chains bind this to the Atmos storage Factory/callback registration path. However, the branch continuation is not inspected and no selected DDP-JOC 5.1 stage, EC3 frame writer, or F13C writer is established. Ghidra renders `FUN_1403f1dd0` as `void`, although its post-call RAX is forwarded; this Wash preserves that dataflow without assigning a return type.

## Disposition

Reference-only supplement; `implementation_ready=false`. The next minimum static edge is limited to the branch target and fallthrough from `0x1403a6984`; stop if they do not reach a typed audio writer. No audio behavior or absence-of-use claim is promoted.

Ghidra static analysis only. No encoder, PCM, payload, audio output, license, HMAC, or key content was accessed.
