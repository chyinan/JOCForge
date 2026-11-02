# Sample 6 DRC core dispatch consumer boundary

Recipient: implementation evidence supplement  
Disposition: bounded static call-path finding; runtime row values remain unbound.

## Verified call path

Wash independently verified the upstream R2 package manifest, seal, all five payload hashes/sizes, and its checker in normal Python and with `python -O`.

For `dee_ddp_encoder.exe` (SHA-256 `0cf23520799f52323b0994e5fcc16924da7b027a5b83a0cff3b4f06792c18532`), the data pointer at `0x140610538` is the `+0x28` entry in the type-indexed channel method table and resolves to `FUN_140350d00`. The process API `FUN_140213470` (`DDP_ENC_PRO_ddpi_enc_process`) reaches `FUN_140315db0`; that dispatcher calls the selected method with the channel instance, frame/work record, frame descriptor/config, and root-owned sample pointer. `FUN_140350d00` then loads the state pointer from that same instance at `+0xdf8` and calls `FUN_140317600`.

The checked pre-dispatch hooks and frame/work-buffer writer do not pass a typed pointer to, or write/copy into, the inline six-by-five F32 row `[state+0x23e34,state+0x23eac)`. This is a checked in-image path boundary; it does not rule out an external writer or an unresolved indirect path. Runtime row values remain `UNBOUND`; constructor zero bits are not runtime values.

## Next edge

The remaining edge is the process-frame input from `FUN_140214e80` / `FUN_140214ef0` through `FUN_140215070` and `FUN_140213470` to a typed writer/copy into that same row before `FUN_140317600` reads it. The package binds the wrapper callsites but not this writer bridge.

The accompanying vector is symbolic and `NOT_RUN`; it tests only the documented call-target/owner relationships. It is not a runtime capture or evidence of F32 contents. No new DEE run, raw PCM/syncframe/payload access, or private-key/license access occurred.

## Upstream seal

- Package: `sample6-matched-drc-core-dispatch-consumer-wash-v1`
- MANIFEST.json SHA-256: `5d3c14fcaa9691b991154a3b5f4eafb79a70c95815c35ee4e413f7a69ca3cdbc`
- package_seal.json SHA-256: `8abdf32b1c647e5dab90bbac171ff3f0d215b3d4c43cb01e29390f3430c61ef9`
- Checker: normal and `python -O` passed.
- Five manifest payloads were independently verified by Wash.
