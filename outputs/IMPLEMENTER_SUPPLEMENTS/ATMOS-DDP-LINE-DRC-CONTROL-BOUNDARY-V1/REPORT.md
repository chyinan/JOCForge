# Atmos DDP Line DRC control boundary

**Status:** conditional Line DRC control-bit path bound; selected job mode unobserved; base decoded PCM effect not determined statically; A/B not run.

## Bound static path

For the hash-bound Atmos DDP plugin, the reported `film_light` input path supplies tree U32 `0`, which the mode-1 branch maps to raw Line DDP_COMPR code `2` at configuration `+0x28`. The value enters parameter ID `0x0d`, is stored in per-stream state `+0x100`, then reaches a presence-tagged control record. The serializer writes the present record's two 3-bit fields into the control buffer.

The observed sample path uses a separate call chain to copy and submit sample arrays. The bounded Ghidra trace found no direct write of this DRC value into those arrays. This supports a separate E-AC-3 control/signaling path; it does **not** prove that the final decoded base PCM or Atmos render is unchanged, because decoder DRC application was not traced and no matched output was decoded for this question.

The job's actual mode-1 branch selection/property submission remains unobserved. The `film_light → code 2` mapping is conditional on the value reaching the descriptor and the mode-1 branch being selected.

## Minimal distinguishing experiment

If needed, compare the same authorized ADM and all other job fields with only `line_mode_drc_profile` changed from `film_light` to `none`. Record the accepted config, descriptor and per-stream values, emitted DRC control bits, and output metadata. Decode with DRC application disabled and compare per-channel hashes/sample differences in memory only. This is a **PROPOSED_NOT_RUN** vector; do not claim an audible or decoded-PCM effect from the static trace.

## Evidence scope

Wash verified the source package manifest, seal, sidecar, and 11 payloads. The package carries 100 source references, including 96 Ghidra/source evidence entries; Wash did not independently rehash all 100 external source artifacts in this pass. The target image hash matches the previously bound Atmos plugin identity. No DEE run, PCM/payload read, key/license read, or implementation contact occurred in the source audit.
