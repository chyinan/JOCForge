# DEE Atmos source axis to OAMDI capacity boundary

## Static scope

Ghidra 12.2 statically binds the selected StageRunner route to `audio.atmos_mezz` → `encode_to_atmos_ddp` and the Atmos DDP plugin. The analysis follows one source/PE branch but does not bind the job's actual `input_adm.wav` identity or runtime source generation.

The inspected AtmosProcessor requires the actual frame channel count to equal its configured count and the frame size to fit its allocated extent; neither check contains a fixed 38/48 limit. Its multichannel buffer allocates N rows and copies the requested sample extent per row.

For the inspected MdToEvo projection, descriptor entries are six bytes; descriptor byte `+2 == 0xFF` is omitted from the active list. If N is physical channel count and E is the retained descriptor count, the companion remainder is `R=N-E`; the OAMDI converter guards `1 <= R+E <= 159` and sizes the source table from that total. The first explicit source-count cap in this traced path is therefore 159. This is not proof that arbitrary ADM metadata reaches this path or that deeper PE/JOC internals accept all such counts.

The OAMDI table has a separate expanded axis: bed/channel-mask presence, layout selector entries, and object entries. The report lists layout additions `{4,8,10,14,15,30}` for selectors 1–6, with a 159-entry capacity. Overflow returns early, but clean caller error propagation was not established. Later profile code-row shapes are mode-specific `{16,12,10,8}` and are not the physical ADM N axis.

## N=36/38 boundary

Static arithmetic shows 36 and 38 are below the OAMDI 159-entry source-count cap and fit the inspected U16 and dynamic row/allocation formulas. No fixed 38/48 source-axis check was found before the opaque PE/JOC API boundary. The authorized N=36 metadata/input identity is absent from this audit; the available N=38 metadata sidecar is not joined to the job's `input_adm.wav`. Neither case is an observed DEE acceptance result.

The useful implementation rule is to keep physical ADM rows and the later profile code-row axis separate. Do not equate physical N with `{16,12,10,8}`. The explicit source-index/physical-host mapping still requires a typed identity rule; the proposal in the report is a model requirement, not an observed DEE mapping.

## Non-closure and disposition

- Host source-property to worker owner/generation is `UNBOUND`.
- Selected job source identity and metadata are `UNBOUND`.
- Runtime 36/38 acceptance and opaque PE/JOC internal limit are `NOT_RUN` / `UNBOUND`.
- OAMDI count-to-writer and writer-to-selected-AU joins remain `UNBOUND`.

This is a static count/shape contract, not an audio-equivalence result. It supports a bounded count-driven 36/38 implementation slice and input-only vectors; it does not certify arbitrary-N end-to-end output.

## Provenance

- Upstream package `DEE-ATMOS-SOURCE-TO-AU-WASH-V1`, manifest SHA-256 `b864c94152763edc68a8d2d595c6e8526acb9af3de2bd7853837f05d4abf79c6`.
- Wash independently matched all 32 manifest payloads and all 7 external references.
- Package reports Ghidra 12.2 static analysis; DEE, decoder, PCM, payload, and key/license access were not run.
- Upstream verifier results are recorded in the package; Wash did not rerun its PowerShell verifier.
- No raw Ghidra logs/scripts, code, media, PCM, or payload are included here.
