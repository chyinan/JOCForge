# F05Q.3 ID14 header/framing bounded coverage

## Covered

- Official writer, bit writer, body-bit builder, native validation order and F05 parent retry call order were statically traced in the selected DLL build.
- 43 isolated native writer invocations include 20 success and 23 failure cases. All 20 successful bytes rebuild exactly from captured inputs and the statically traced MSB-first rules.
- Three live writers from one unmodified full-song encode were captured. Rebuilt header bits match actual payload bits; byte length equals ceiling(valid_bits/8).
- Reference EC3 track is 4,247 frames / 17,395,712 bytes and matches the accepted baseline. No observer errors.

## Remaining bounded gaps

- All semantic meanings of object kind and mode enums are not established; only wire widths, range checks and selected values are known.
- The full upstream mapping from model objects to descriptor values across alternate modes/profiles is not established.
- Retry fault injection and rollback of every enclosing JOC state field are not run.
- No absence claim is made for unexecuted profiles or other builds.

## Classification

- `NATIVE_OBSERVED`: writer and bit-writer operations and caller retry flow from official DLL static analysis.
- `RUNTIME_CONFIRMED`: isolated writer cases and first three full-song writer calls.
- `MODEL_POSTCOMPARE`: `NOT_RUN`.
- `STATIC_ONLY` / `NOT_OBSERVED_IN_BOUNDED_COVERAGE`: enum meaning and alternate producer/retry behavior.
- `PRIVATE_OR_NON_DELIVERABLE`: no private auth; no raw PCM/history/ADM metadata.
