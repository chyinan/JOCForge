# Active JOC sample-boundary quantization

## Narrow rule accepted

For the selected 38-channel ADM BWF / active JOC writer path, keep the validation helper and final writer conversions separate:

- `SampleRange` rounds `sample_rate * seconds` for source-bound and nonempty-range validation.
- The active JOC composite writer converts each already-normalized positive time boundary to a signed 64-bit sample index by truncating `seconds * sample_rate` toward zero.

For delimiter-free numeric tokens at the selected 29.97 override, the source rule is `frame_index / 30 * 1001 / 1000`; a nonzero numeric end token is incremented by one frame before conversion. Run-53's one-shot active-writer capture and native log match the truncated final endpoint. This supersedes the earlier general nearest-round interpretation for final JOC writer boundaries.

## Verified vector

With file-position time base, filter rate `29.97`, 48 kHz, start token 2160, and inclusive end token 2162:

- start time = 2160 / 30 × 1001 / 1000 = 72.072 s; product = 3,459,456 samples.
- exclusive end time = 2163 / 30 × 1001 / 1000 = 72.1721 s; product = 3,464,260.8 samples.
- validation uses rounded bounds `[3,459,456, 3,464,261)`.
- final active JOC writer fields are `[3,459,456, 3,464,260)` (length 4,804), matching the reported native log and dynamic post-return field capture.

The 24 fps vectors have integral products and remain valid; they could not distinguish rounding from truncation. Embedded-timecode static logic subtracts the configured origin before truncating the final products, but no fractional embedded-timecode runtime vector was tested here.

## Evidence and limits

Wash verified v4 manifest/payload hashes, its ordinary and optimized verifier, and its v3 parent manifest. The source package records a direct run-48 native log boundary and a run-53 active-writer entry/return capture; raw debugger transcript, logs and binaries were not included or rehashed by Wash. The source-rate/runtime capture relationship is hash-bound to the package's observation extraction, not independently replayed. No raw PCM/audio values were read or persisted. Model output and comparison remain `NOT_RUN` and `NOT_EVALUATED`.

This rule is limited to the selected active JOC writer and the documented numeric-frame conversion paths. Colon/drop-frame parsing, other fractional rates, negative/halfway products, EOF, multi-row, append/prepend and model/native comparison remain open. Do not use `SampleRange`'s rounded values as the final composite-item boundaries.
