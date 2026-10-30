# Selected length boundary coverage across synthetic noise inputs

## Scope

This is evidence for the selected profile and three separate native captures: run-100's authorized source input, run-109's 30-second generated-noise prefix with a zero-filled tail, and run-110's full-duration generated noise. Each run independently reports a same-parent count-to-body-length chain. The runs are not merged into one execution identity.

## Repeated native relation

In each capture, the selected body length joined to the planned, serialized, refill, and U16 body-prefix length for every reported same-parent count event. Block-2 wire and body-region reservation values matched body bytes plus 4; the 9-bit writer operand matched those values and advanced 9 bits.

Observed ranges:

| Run | Same-parent events | Body length | Block-2 operand |
|---|---:|---:|---:|
| run-100 source input | 4,247 | 232～375 bytes | 236～379 |
| run-109 noise prefix | 938 | 297～440 bytes | 301～444 |
| run-110 full noise | 4,247 | 297～413 bytes | 301～417 |

The repeated relation supports this selected-path observation across the listed inputs. It does not name the meaning of `+4` or establish behavior outside these observed ranges.

## Remaining boundary

Operands 507, 508, 511, and 512 were not observed in the listed captures. The 512 overflow outcome remains unknown. Run-99 upstream threshold/candidate/pretrim fields still lack units and a complete formula. Parent invocation to ADM sample position remains `UNBOUND`. Model codec execution and JOCForge runtime remain `NOT_RUN`.

The synthetic runs report that source PCM bytes were skipped without reading or copying them. Their EC3 outputs are synthetic-input results, not comparisons against the original source-audio output.
