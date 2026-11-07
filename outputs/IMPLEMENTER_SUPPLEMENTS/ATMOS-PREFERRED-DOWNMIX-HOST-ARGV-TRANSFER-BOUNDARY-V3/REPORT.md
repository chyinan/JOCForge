# Preferred downmix host-to-child argv boundary

## Static host path

The hash-bound DME app-source trace carries the active `preferred_downmix_mode` property in the composite `--encoder` value through the host configuration and process-launch path:

`PropertiesMapper.map()` → `Cmd.encoderConfigProps` → `ServerJobFactory` → `ServerJob.config` → `generateCmd()` (push key, then value) → `DeeCaller` (copy argv) → `ShellCmd` → `child_process.spawn(binaryPath, args, ...)`.

Within this static host path, the key/value pair is preserved in the composite argument; no host-side split, rename to `dmixmod`, or drop is observed. This does not show what the native provider parser does with the composite argument.

## Conditional provider selection

For the DDP Online Media encoder, the app source selects `dee_ddpjoc_encoder.exe` on Windows when the DME input value is `ChannelBasedImmersive` or `DolbyAtmos`; other inputs select `dee_ddp_encoder.exe`. The prepared `atmos_mezz` XML candidate is not a runtime DME `input` capture, so the actual tool branch remains `UNBOUND`.

## Exact remaining edge

The static host-to-child argv path is bound. The edge from that argument to the selected DEE provider's native config parser remains `UNBOUND`; native receipts show a distinct `dmixmod` key/allowlist but no accepted/split/renamed/dropped handling of the composite `preferred_downmix_mode` member. Runtime UI/inputType, actual child argv, native typed owner, audio/BSI effects, and LoRo/LtRt A/B remain `NOT_CAPTURED`/`UNBOUND`/`NOT_RUN`.

## Provenance

Upstream manifest SHA-256 `8a36ee08e5e500feee68c133e54e9cc910735677b31afaf539af2efcc16f6bf6`; upstream SHA256SUMS file SHA-256 `3b074606eadcc8e96617e77347496a5d6bd280332d1ca85a0ba67a43a44ef17e`. Upstream normal and `python -O` checks passed over five payloads/six checksum entries and eight parent receipts. The `app.asar` hash was recomputed in the source pass. This supplement contains no raw app source, script, native binary, media, PCM, payload, key, or license.
