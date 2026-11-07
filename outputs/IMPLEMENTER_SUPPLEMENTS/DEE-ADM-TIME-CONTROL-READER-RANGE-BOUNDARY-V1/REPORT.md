# DEE ADM timeline controls to DDP reader-range boundary

## Bounded time rules

The hash-bound R138 input uses time_base=embedded_timecode, start/end tokens 86640/86664, rates not_indicated, offset=auto, and ffoa=auto at 48 kHz. In the embedded_timecode branch, numeric start/end are parsed as frame counts, divided by the resolved frame rate to seconds, then rounded to sample indices. Offset rebases the source bound and the converted endpoints. Because the R138 resolved frame rate was not captured, no concrete sample indices are asserted.

FFOA is not observed in the explicit embedded_timecode start/end branch. The relation max(offset, ffoa)-offset belongs to a separate file_position branch; do not apply it to the R138 embedded_timecode case.

The host normalizer's typed argument order is map, timecode_frame_rate, ffoa, offset (RCX/RDX/R8/R9). The DDP Atmos time setter copies a 15-qword record: slot 10 is binary64 offset seconds and slot 11 is binary64 FFOA seconds. Its input-record writer, selected receiver owner, and R138 invocation remain unbound.

## Stage and reader path

Static StageRunner evidence selects encode_to_atmos_ddp, creates the DDP stage, and sends a host source-property packet to its property handler. No evidence shows that packet contains the normalized ADM start/end/offset/FFOA map. The selected DDP range path is statically traced through sample-range conversion to the multi-source IO set and encoder setup, but its runtime range-vector owner/generation and specific reader request were not captured.

The host ADM object and DDP range path are individually bound; they are not yet joined through one R138 run. This supplement does not claim runtime timing parity or a specific rendered excerpt.

## Limits

DEE/runtime was not run. No media/PCM/EC3/key/license contents were read. The authorized source metadata hash is recorded as an input-only identity; the timeline fields are not treated as a native runtime receipt.
