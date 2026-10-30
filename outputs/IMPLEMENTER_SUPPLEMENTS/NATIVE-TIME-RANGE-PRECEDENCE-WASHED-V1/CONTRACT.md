# Bounded embedded-timecode offset precedence

## Rule accepted for this fixture condition

For the selected 38-channel ADM BWF fixture, delimiter-free numeric frame tokens, `time_base=embedded_timecode`, 24 fps, 48 kHz, and an explicit source `offset` while FFOA is `auto`, the explicit offset replaces the auto-FFOA fallback as the embedded-time origin.

The bounded conversion is:

- `absolute_start = start_frame / fps`
- `absolute_end_exclusive = (inclusive_end_frame + 1) / fps`
- `file_start = absolute_start - explicit_offset_seconds`
- `file_end_exclusive = absolute_end_exclusive - explicit_offset_seconds`
- For this exact vector, the sample-boundary products are integral; this vector does not exercise fractional-sample quantization.

In run-47, `offset=01:00:11:00` is 3611 seconds; FFOA is auto and the fixture's transcribed auto value is 3610 seconds. Tokens 88824 and 88826 therefore produce absolute times 3701 and 3701.125 seconds. The native record is `[4,320,000, 4,326,000)`, matching the explicit-offset calculation. Using the auto-FFOA value instead would start at sample 4,368,000. The static loader path reads FFOA as fallback and replaces it when an explicit offset is present.

Wash permits implementation of this bounded precedence branch in the selected ADM input/time-range reader: for the selected profile's `embedded_timecode` input with an explicitly authored offset and `ffoa=auto`, use the explicit offset before applying the frame-index/origin conversion. The verified case is offset `01:00:11:00`, 24 fps, 48 kHz, and tokens 88824/88826. This approval covers that source-selection branch and exact vector only; both sample boundaries are integral, so no fractional-sample quantization rule is authorized. It is not a general approval of all offset strings, timecode syntaxes, or frame-rate modes. It belongs in the existing selected ADM input/range path and is unrelated to the F03U source-weight path.

## Limits

This closes only the observed explicit-offset/auto-FFOA condition on this fixture. It does not establish precedence when both fields are explicitly authored, other ADM metadata combinations, other frame-rate modes, colon/drop-frame syntax, EOF, multi-row or append/prepend behavior, or model/native equality. Fractional-sample boundary quantization is currently `UNBOUND`: a later run-48 report from Agent 1 conflicts with the prior general round-to-nearest wording, and its hash-bound evidence package is still pending Wash review. Do not implement a general quantization rule until that review completes.

The frame rate and auto-FFOA value were transcribed from the run-43 observation; its raw debugger transcript was not persisted. The run-47 range is reported from the native log, but the raw log is not included. The three output artifact hashes match according to the source package; that establishes encoded artifact identity only, not PCM wordwise equality. Model output and comparison remain `NOT_RUN` and `NOT_EVALUATED`.
