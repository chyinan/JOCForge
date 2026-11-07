# DEE ADM type-1 sampler record/state to metadata boundary (current)

## Disposition

This supplement supersedes `DEE-ADM-TYPE1-SAMPLER-RECORD-STATE-EDGE-V1`. Wash verified the source package manifest, four payloads, SHA256SUMS, package seal, parent type-1 and float-state packages, and current target plugin image.

## Same-slot readback and conditional float state

Within one `FUN_1801ad9d0` slot iteration, the type-1 writer dispatch and subsequent readback both use the same current per-slot pointer. `FUN_1801b6fa0` reads back the same nested receiver's 0x58-stride descriptor records and dirty flags written by the type-1 sampler handler. Its conditional `+0x80` state gate can call `FUN_1801caa90`, which performs a float/state recurrence and updates the nested state; no specific input branch execution was observed.

## First downstream consumer

`FUN_1801a7710` passes context field `+0x858` to `FUN_1801ad9d0` as the row-output table. `FUN_1801ad9d0` writes active 0x58-byte row descriptors to that table. It then passes the same context field to `FUN_1801acd10` as argument 4. This is the first direct consumer: it reads the 0x58-byte descriptors and produces a separate array of 0x34-byte canonical metadata records.

The handoff is explicitly frame-metadata processing, not a demonstrated PCM or AtmosProcessor audio-block call: the callsite has no PCM/audio-block pointer. Therefore the current static path closes the conditional sampler state through canonical metadata materialization, but not audio samples or encoded output.

## Limits

- The `+0x80` float/state branch is conditional and was not observed for any specific ADM.
- No runtime generation token, PCM/audio-block transfer, encoded-output handoff, or downstream use of canonical records was captured.
- No DEE run or media/payload/key/license read; no map/type-table reanalysis.

## Implementation scope

Use this as a bounded metadata contract only: same-slot descriptor update/readback feeds a canonical metadata record consumer. Do not treat the float recurrence as PCM/DSP until an audio-block edge is bound. The next exact edge is the first consumer of the canonical 0x34 record array.

## Source package

`C:/Users/chyinan/Documents/Codex/2026-11-01/jocforge-reverse-agent-5-f09-u5/outputs/adm_sampler_audio_edge_first_consumer_wash_v1/`; manifest SHA-256 `01e82938e5d958dabe2786e3e465ac719107d95d03f2bf50bbf61be053fdaa2b`; package seal SHA-256 `537ef0927c60cc54bbf3a30c3b62c63cc909444438b3931fa6a0ff658bf4c18a`.
