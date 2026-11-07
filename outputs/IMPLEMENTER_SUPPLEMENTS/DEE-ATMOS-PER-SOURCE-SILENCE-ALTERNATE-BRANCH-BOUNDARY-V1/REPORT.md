# DEE per-source silence vectors in the alternate AtmosReader branch

## Bounded branch finding

In the selected `FUN_180175c80` branch, the four per-source vectors reach `FUN_180177230`, which validates vector lengths against the source count and passes their values to `FUN_1801844c0` to build temporary composite parameter rows. In the traced flow, `FUN_180185240` then constructs the reader wrapper from separate inline context fields and fixed parameters, and releases the temporary rows. No copy from those four row values into that reader's configuration was found.

The corresponding reader chain is `FUN_1801850a0` → vtable slot 3 `FUN_180186520` → `FUN_180186870`; the window read fills data pointer/channel/sample fields without consuming the four duration vectors in this bounded trace. Thus this branch closes the vector-to-parameter-row construction but does not show per-sample silence, cursor, or reader-limit behavior from the vector values.

## Do not join sibling worker by shared semantics

`FUN_180179ab0` is a separate 1536/flush worker. Its direct owner is `FUN_180179a80`, and its call references are in the CBI workflow `FUN_180175500`, not the selected `FUN_180175c80` call chain. This package does not join those branches or their owner/generation. The separate worker's flush behavior is not evidence that the per-source duration vectors insert samples or change its flush count.

## Status

- Selected branch: vector lengths and temporary parameter-row construction are bound; vector values to reader/core sample consumption are not found in this branch.
- Other encoder branches and global consumers: `UNBOUND`.
- AU count and one-span/one-AU behavior: `UNBOUND`.
- Runtime behavior: `NOT_RUN`.

This addendum narrows the earlier silence-vector correction to a second branch. It does not assert that silence is ignored by all DEE paths or that the existing generic reader preclear creates requested silence.

## Provenance

- Upstream `dee_atmos_ddp_audio_window_consumer_wash_v1`, manifest SHA-256 `f43b90771b2a4982f8d52b1657593519db5990243832acc846636f629b547d14`, package seal SHA-256 `a07d42333921564ea318833e1b2b10cc21754d06b0811ebe7a608b477ea52145`.
- Wash independently recomputed all 26 payload hashes and manifest/seal linkage.
- Ghidra 12.2 static only; no DEE execution or PCM/payload/key access.
