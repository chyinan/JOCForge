# Speech-threshold meter consumer boundary

## Wash disposition

The upstream addendum narrows the integer threshold to a caller-owned meter configuration slot and the first named `RunMeter` stage. It does not show that the slot is read by the meter object or compared with a speech score.

## Typed path

- The `measure_only` branch calls the metering handler with base `lVar16 + 0xd40`.
- The handler writes `speech_threshold` into `param_1[2]`, resolving to `lVar16 + 0xd48`; DI is stored separately in `param_1[1]` and does not gate threshold assignment in this handler.
- The mode-2 `measure_only` path proceeds through `FUN_1801ec120` / `FUN_1801ec690` (`RunMeter`) and calls the meter object. The scoped static exports do not show the `+0xd48` value copied into or read by that meter object.

## Boundary

The first speech-score comparison, operator, equality behavior, rounding, and formula remain `UNBOUND`. A separate `measure_and_correct` workflow reads an offset `+0x590` from another owner, but there is no typed owner/generation join to this `measure_only` slot; do not infer identity from the offset. No evidence connects this threshold slot to PCM/spatial DSP. Do not implement a comparison formula from this receipt.
