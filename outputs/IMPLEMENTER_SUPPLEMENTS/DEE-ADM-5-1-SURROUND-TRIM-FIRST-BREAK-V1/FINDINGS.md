# 5.1 Surround Trim First-Break Boundary

## Scope

This washed handoff records one static boundary for `surround_trim_5_1`; it does not supply a trim algorithm or claim a PCM effect. The source package was independently verified in the Wash workspace. Its checker references a raw Ghidra log that is deliberately not included here; this supplement contains only sanitized findings, hashes, and an input-only vector.

## Bound facts

- The XML metadata declares `surround_trim_5_1` as an `xs:string` enum `{auto, 0, -3, -6, -9}`, default `auto`; it defines no unit at this boundary.
- One existing input-only vector contrasts `auto` with `-6`, holding `height_trim_5_1=auto` in both cases. That vector was not run in the source package.
- A single candidate function reads a U16 at numeric displacement `+0x968` and copies it into a record. The key-bound trim StringType uses the same displacement, but the candidate object's owner is not typed as that StringType.

## First break

The XML StringType value has not been joined to a same-owner numeric trim consumer. Unit conversion, rounding/clamping, downstream per-channel gain, and PCM effect are all `UNBOUND/NOT_RUN`. Do not equate the U16 candidate with the trim from offset equality. No implementation rule can be derived from this package.

## Exclusions

No raw Ghidra/decompilation source, binary, media, PCM, encoded payload, key, or license content is included. No DEE run or new A/B occurred.
