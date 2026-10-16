# Callback output persistence amendment

## Status and source identity

This document is an approved wash-side amendment to the logical F03W state schema. It defines `selected-adm-installed48-state.v4` and supersedes only the callback-output persistence statements in source schema `selected-adm-installed48-state.v3`. The approved package is immutable: `parameters/adm-installed48-state-v1.json` remains v3 evidence and is not relabelled or edited.

The v3 package state file is 7,237 bytes with SHA-256 `5138318de304b83d26e47271c1661bcc903cebcd2cea457ae98e0048b161ed46`, under package MANIFEST SHA-256 `c7cddb7d5ac073c0f410fe44be9a431cfbb21c3df469d8b43c32a4297298674a`.

Wash-side amendment basis:

- Findings SHA-256: `cf25b05789219c08a02d992032d32bd1cb2822b691a133ce185533a2f2555a1b`
- Input-only vectors SHA-256: `aea2a6286ba523b7867e474410e44b4a88f1011783c5252f9bcdd98ca24b6678`
- Research result SHA-256: `fa83260a2e6075e1fb81a439deb7060d715ffa08cfafea352902d608c4bfa4f4`

This evidence closes only the two callback-output persistence classifications below. It does not establish product-root, transaction, full first-callback, input/FFT, or end-to-end audio execution. Those remain `NOT_RUN`; no key or DLL evidence is required by this amendment.

The washed evidence reports that the Python runtime has no independent persistent `current_flags` or `current_features` fields. A 144-row RESET/CARRY matrix confirms the feature packet matches the staged-after source vector transpose bit-for-bit. Two 48-row flags probes show identical before history can yield different flags when the current callback inputs or writers differ, so flags cannot be a fixed cold array.

## Persistent root changes

The v4 persistent `f03w_owned_fields` does not contain `source_flags.current_flags` or `feature.current_features`.

- `source_flags` has no independent persistent bytes. Omit the group from the persistent root canonical form; do not emit an empty group, presence marker, or other tag. Its v3 `current_flags: U8[48][2]` description is retained only as source evidence for the callback output packet below.
- `feature` retains `source_vector: F32[48][24]` and `source_scalar: F32[48]`. Cold/reset values for both are the positive-zero bit pattern `0x00000000`. `current_features: F32[24][48]` is a callback output, not persistent root state.
- The remaining v3 persistent fields, cold constructors, field order, shapes, and canonical encodings remain as previously mapped, except for these two removals and the v4 packet boundary. Callback-local scratch and external owners remain excluded.

Both removed fields contribute exactly zero bytes to the complete v4 root canonical preimage. There is no absence/presence tag for them. Recompute the complete root serialization and SHA-256 from the v4 persistent fields; do not adjust any v3 digest by subtracting 4,704 bytes.

## Callback output packets

`SourceFlagsPacket` has `callback_index = g` and `flags: U8[48][2]`. Derive it from the same callback's before history together with that callback's current input, control, route, and prefix writer set. It is not persistent state, is not a next-callback seed, and contributes no root digest bytes.

`CurrentFeaturesPacket` has `callback_index = g` and `features: F32[24][48]` in band-major order. Its values must match the same callback's staged-after source vector bit-for-bit:

```text
features[band][source] = source_vector_after[source][band]
```

It is not a second persistent history and contributes no root digest bytes.

Both packets belong to the same parent `CallbackTransaction` as their corresponding staged-after root. On parent success, commit only the true persistent histories; packet storage remains outside the root. On parent failure, invalidate the packets and leave the committed owner root unchanged. A packet from a failed transaction is never a recovery authority.

Cold/reset creates no packet; a packet read returns `NOT_READY`. If a read-only cold feature view is required, it may be derived by transposing the positive-zero source vector, but this does not mean callback 0 has completed. On the next callback, recompute flags from the then-current before history and inputs; read features from committed `source_vector`/`source_scalar`, never from an older packet. Diagnostic copies may be recorded externally but are not restore inputs.
