# R138 process-state input-map owner boundary v2

## Bound static path

Ghidra evidence binds the selected StageRunner descriptor name `encode_to_atmos_ddp`, its factory dispatch, and product storage on the same Runner expression. The product's `+0xb8` member resolves to one process-state object. The descriptor's context at `+0x138` is passed through the filter initializer and copied as 0x20-byte entries into process-state `+0x08`.

The `+0x08` vector is not typed-joined to the distinct input metadata map at process-state `+0xA0`. That map contains WAV-shaped fields (`filename`, `sampling_rate`, `samples`, `channels`, `offset`) and feeds the channels reader. Processing routines populate the separate output/normalized map at `+0xF88`; the channel path then writes `+0xFC8`. The origin of `+0xA0` for the R138 ADM/StageRunner input remains unbound.

The R138 job schema contains no channel-named XML elements. This is structural absence only; do not infer zero or an implicit channel count. The large offset sweep did not identify an owner-specific writer and is not used to assign one.

## Limits

- **CLOSED_STATIC:** selected descriptor → factory → process-state `+0xb8`; descriptor context `+0x138` → process-state `+0x08`; `+0xa0` is a distinct input map; `+0xa0` to `+0xf88` processing and same-state `+0xfc8` write.
- **UNBOUND:** `+0x08` vector → `+0xa0` input-map transfer; R138 ADM input → `+0xa0` writer; final `+0xfc8` value and custom-BSI applicability.
- **NOT_CAPTURED:** native runner address/generation.

No DEE run, media, PCM, key, or license was accessed. This is metadata ownership evidence, not an audio coding rule.
