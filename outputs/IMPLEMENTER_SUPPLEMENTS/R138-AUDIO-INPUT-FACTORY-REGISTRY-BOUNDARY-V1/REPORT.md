# R138 host audio-input factory registry boundary

## Static path bound

The hash-bound Ghidra package establishes the generic host path: the `audio_input` category maps to `dlb_phoenix_get_audio_input_handle`; `LoadOne` resolves the category getter and invokes the returned local callbacks; callbacks enter the shared input registry vector at manager `+0x18`; StageRunner enumerates candidate configs into its vector at `+0x2F0`; and the schema builder exposes `audio.input.type`.

This closes the generic registry/container path only. It does not identify which module supplies the successful R138 `audio.adm` factory, which local callback slot is selected, which candidate index is chosen, or which typed reader owner/vtable is used.

## Bounded negatives and exclusions

Seven hash-bound candidate images were scanned for the exact getter export and had zero matches. This is not a complete successful-run module inventory. Three prior R138 metadata receipts/logs identify `audio.adm`, the output profile `encode_to_atmos_ddp`, and input stage/storage names, but not the provider image, callback slot, or reader owner. The host `audio.adm` string at RVA `0x2D4968` belongs to the output-side `AtmosMezzPlugin` path and is not evidence of input selection. A failed setup log is not treated as a successful run's module list.

## Disposition

Reference only; `implementation_ready=false`. Keep the selected R138 factory/module/hash, local callback slot, StageRunner selected candidate, typed reader owner/vtable, and effective successful-run plugin roots `UNBOUND/NO`. The next minimum observation is one successful R138 metadata-only setup trace joining those pointer/type tokens in the same run, without reading media bytes. It remains `NOT_RUN` and this package authorizes no new encode.

No new DEE run, license-content read, media-content read, IDA analysis, PCM read, or payload read occurred. The package contains summaries and hashes rather than raw decompilation.
