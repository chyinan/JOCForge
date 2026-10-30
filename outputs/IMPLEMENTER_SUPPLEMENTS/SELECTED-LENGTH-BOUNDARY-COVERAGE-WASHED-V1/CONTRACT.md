# Selected length boundary coverage and next fixture

## Existing coverage

The verified captures cover three input conditions for the selected profile. Run-99 and run-100 used the authorized 38-track ADM BWF with original source audio. Runs 109 and 110 retained that ADM metadata and substituted independently generated noise PCM; the source PCM was skipped without reading or copying it.

| Capture | Same-parent events | Body length | Block-2 operand | Target values observed |
|---|---:|---:|---:|---|
| run-99 | 4,247 producer calls | upstream unit not established | not fully joined | 507/508/511/512 not observed in captured producer fields |
| run-100 | 4,247 | 232～375 bytes | 236～379 | none |
| run-109, 30-second noise prefix | 938 | 297～440 bytes | 301～444 | none |
| run-110, full-duration noise | 4,247 | 297～413 bytes | 301～417 | none |

In run-100/109/110, same-parent body-length prefix/refill relations were complete. Block-2 reservation and writer operand equaled body bytes plus 4, and the writer advanced 9 bits. These are observed relations within each capture; they do not identify the meaning of `+4` or prove boundary behavior.

The authorized metadata inventory reports one selected 38-track source with 28 object packs and 10 direct-speaker packs; six other listed sources have 36 tracks, 26 object packs, and 10 direct-speaker packs. The selected 38-track source is the richest listed candidate for the 15-object route and was already used in run-99. This inventory does not prove any of these inputs can reach the target body sizes.

## Next fixture requirement

The smallest useful new fixture would retain a valid selected-profile 38-track ADM graph and its admitted metadata, while using independently generated PCM. Its actual producer inputs must produce same-parent body lengths 503, 504, 507, and 508 bytes; capture the resulting block-2 operands 507, 508, 511, and 512, along with writer advance, return status, and downstream serializer/refill status.

Those body sizes are requirements for a future fixture, not expected native outputs. The experiment is `NOT_RUN`. A fixture is evidence only after the selected producer inputs and native acceptance are validated. The 512 case exceeds unsigned 9-bit representation; native truncation, rejection, rollback/no-commit, and downstream behavior must be observed directly.

No global-unreachability claim follows from the current no-hits. Parent invocation to ADM sample position remains `UNBOUND`. This package does not read or persist source PCM, encoded bytes, or keys, and does not prove JOCForge output.
