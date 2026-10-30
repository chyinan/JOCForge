# Native reader count, callback span, and padding

## Run-121 scope

Run-121 is one native execution using the authorized ADM BWF. The runtime input hash matched the authorized source. The package reports a native composite range record with fields `start=0` and `end=49,999`; their units and endpoint convention remain `UNBOUND`.

In that same execution, each of two readers made 33 provider calls. Each reader's direct count vector was 32 reads of 1,536 sample frames per channel plus one tail read of 847, totaling 49,999 sample frames per channel. The cursor values 0, 49,152, and 50,688 are reader-relative offsets, not absolute sample positions.

Before each read, the native path zero-filled 38 channel planes of 1,536 float elements each. On the tail call, 847 elements per plane were read and the remaining 689 elements per plane stayed zero. This yields a padded 1,536-frame callback span. The downstream consumer received 33 data-frame calls of 1,536 frames and 5 zero-span flush calls; zero-span calls are not source samples.

## Boundary

The reader range record and reader-state call boundary are joined within run-121. Absolute sample-position origin and whether `end` is inclusive/exclusive remain unknown. The earlier run-115 package remains unchanged and is a separate execution; equal source/output hashes do not establish cross-run identity. No native per-callback generation or sample-position-to-spatial-generation join was captured.

The package contains no PCM or DEE output payload. Model execution and uninstrumented-output comparison are `NOT_RUN`.
