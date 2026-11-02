# Atmos DDP measurement-only dialogue metering inputs

## Wash disposition

The upstream Ghidra 12.2 static receipt binds the two XML values to the `measure_only` metering-parameter path. It supports field types, defaults, bounds, and the handler's input structure. It does not establish the speech-threshold comparison formula or same-run cross-module owner identity.

## Bounded contract

For `encode_to_atmos_ddp/loudness/measure_only`:

- `dialogue_intelligence`: boolean, default `false`; schema text scopes it to manual preset and says it is ignored for metering modes `1770-1` and `LeqA`.
- `speech_threshold`: integer in inclusive range `0..100`, default `100`; schema text scopes it to manual preset.
- The config loader supplies these defaults. The Atmos plugin path writes a bool and integer metering slot. The DDP handler selects `measure_only` mode 2 and processes those slots in the metering parameter handler.
- The prior F06 measurement-result-to-DialNorm path may make changed metering results visible in output metadata. This receipt establishes no direct field path into PCM encoding or spatial transforms.

## Limits

The bottom-level speech score and threshold comparison formula remain `UNBOUND`. Whether every non-manual or metering-mode combination discards the values is only partially supported by schema text and an ignored warning branch. The sibling `measure_and_correct` branch is separate and is not covered by the `measure_only` conclusion. Cross-module runtime owner identity is `UNBOUND`; no runtime, PCM, payload, key, or license material was accessed.

An implementation may model the typed metering inputs and defaults within its existing measurement request. Do not invent the threshold formula or connect these fields to PCM/spatial DSP from this package.
