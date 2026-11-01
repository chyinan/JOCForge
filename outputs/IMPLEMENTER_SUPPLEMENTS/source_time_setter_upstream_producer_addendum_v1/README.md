# Source time setter upstream-producer addendum

Recipient: Wash only. Implementation delivery is prohibited.

This addendum closes the static meaning of two fields written by the source-master setter and defines the smallest read-only observation that could bind their producer to the R138 invocation.

## Static facts

- The setter accepts a pointer to a 64-bit field record. Its body copies slots 0 through 14, so the record must be readable for at least 120 bytes at entry.
- Slots 10 and 11, at byte offsets 80 and 88, are copied into two master fields. The later getter/read adapter treats those fields as binary64 seconds and normalizes them as `offset` and `ffoa`.
- The setter is guarded by a master mode byte greater than 1.
- The setter is present in the master type's virtual table. Ghidra found no direct code call reference to the setter. A module-wide scan found 156 generic calls through the same virtual slot; their receiver identity is not bound to this master object.

## R138 binding status

R138 is bound to the authorized sample-6 ADM BWF hash and to the embedded-timecode configuration hash in this package. The source profile sets the timebase to `embedded_timecode`, gives numeric start/end tokens, and leaves both configured timecode-rate fields as `not_indicated`. The resolved frame rate is `NOT_CAPTURED`.

The storage-master setter's caller, input-record writer, master-object identity, and invocation in R138 remain `UNBOUND` or `NOT_CAPTURED`. The static fact that the two fields are later named `offset` and `ffoa` does not bind them to the R138 source configuration or to the embedded-timecode range calculation. The native R138 observation that offset and FFOA are equal for this sample does not identify their writer.

## Minimal observation if the edge is still needed

After reviewing the existing R4 receipt, use one bounded R138-profile run with the authorized sample hash. If DEE requires ASCII paths, make a byte-for-byte copy in that run's isolated input directory, then verify and record both hashes before launch. Do not change or overwrite the original.

At the setter entry, verify the loaded processing-image hash and record only:

1. The R138 job/config/input identity; a token for the master object; an explicit generation token if one exists; the mode guard; caller return address; and bounded call-stack token.
2. The source-record pointer's readable extent through byte 119, without dumping that record.
3. Only the 64-bit values at byte offsets 80 and 88, interpreted separately as binary64 seconds.
4. The same master's post-write getter values and the subsequent normalized `offset`/`ffoa` reads, joined only by the same explicit master-owner and job/generation tokens.
5. The embedded-timecode resolver's resolved frame-rate value as a separate field. Do not derive it from start/end tokens, sample indices, event order, or either seconds value.

If the setter is not hit, record `NOT_HIT_FOR_THIS_R138_RUN`; do not infer global absence. If no explicit generation token is available, leave that join `UNBOUND`. Capture no PCM, full source record, or payload. No new native job or ASCII copy was created for this static-only turn. Other authorized ADM files were not used because they cannot resolve this input-independent static caller/writer gap.

## Current disposition

`setter_input_producer=UNBOUND`; `setter_callsite=UNBOUND`; `R138_setter_invocation=NOT_CAPTURED`; `resolved_timecode_rate=NOT_CAPTURED`; `model_replay=NOT_RUN`; `Gate1_closed=false`.
