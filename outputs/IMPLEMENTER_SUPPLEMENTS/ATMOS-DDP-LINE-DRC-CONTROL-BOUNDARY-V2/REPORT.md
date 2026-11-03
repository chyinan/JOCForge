# Atmos DDP Line DRC control boundary

**Status:** conditional Line DRC control-bit path bound for `film_light` and `none`; selected job mode is unobserved; base decoded PCM and Atmos-render effect remain unbound; A/B not run.

## Bound static values

In the bounded Ghidra 12.2 path, the DLL-local profile tree maps `film_light` to tree U32 `0`, then to DDP_COMPR code `2`; `none` maps to tree U32 `5`, then to DDP_COMPR code `0`. Both values are staged as an explicit parameter ID `0x0d` record and copied to per-stream state `+0x100` when the record is present. The value `0` for `none` is **not** treated as an omitted parameter record.

If the separate guarded serializer path is entered, its control-record presence flag is set unconditionally and the Line value is serialized as three bits: `film_light` code 2 → `010`; `none` code 0 → `000`. Whether this job enters that outer guard remains unobserved.

The sample-array path is separate: the bounded trace finds no direct DRC write into sample arrays. That does not establish decoded-PCM equality, because decoder DRC application was not traced and no decoded comparison was run.

## Minimum test if needed

The proposed one-field A/B keeps the same ADM and job settings while changing only `line_mode_drc_profile` from `film_light` to `none`. It records the accepted config, per-stream field, presence bit and serialized control code; a DRC-disabled decode compares only per-channel hashes and sample-difference summaries, with PCM held in memory. This remains **PROPOSED_NOT_RUN**. The job's selected mode and audible effect are not claimed.

## Evidence scope

Wash verified the source package manifest, seal, sidecar, and 11 payload hashes. The source package lists 100 source references; those external source entries were not all independently rehashed in this Wash pass. No DEE run, PCM/payload read, key/license read, or implementation contact was performed for this static audit.
