# Native-confirmed lowercase drop-frame branch

## Implementation scope approved by Wash

Implement the valid lowercase `df` timecode branch for the selected supported 29.97 file-position mode. The parser suffix is exactly lowercase `df` at the end of `HH:MM:SS:FFdf`; use nominal 30 labels and the 1001/1000 scale, remove two labels each minute except every tenth minute, and use colon end components directly as time positions (no numeric-token `+1`). The run-79 native vector confirms `00:01:00:02df` through `00:01:00:12df` maps to `[2,882,880, 2,898,896)` and successfully completes the reference encode.

The static grammar/validation contract may be implemented within that branch. The exact native observed input/range is a required regression vector. This approval is limited to the selected `29.97` + `file_position` mode; it does not authorize semicolon/drop-frame punctuation, uppercase `DF`, other rate tokens, embedded-timecode `df`, or general model/native output equivalence.

## Evidence boundary

The reverse source package marks itself `implementation_delivery=PROHIBITED` because it is Wash-only. This V2 is the sanitized Wash disposition for implementation and supersedes the conflicting V1 status wording; it contains no raw research material. Run-79 completed normally and its native composite range matches the static prediction. The debugger only launched/resumed with exception filters; no code/data breakpoints or target-memory reads occurred. Model output, output-interference comparison and stream-profile probe remain `NOT_RUN`/`NOT_EVALUATED`. Whole-chain readiness is unchanged.
