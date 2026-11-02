# DEE StageRunner AtmosMezz to DDP Atmos factory binding

Recipient: implementation evidence supplement  
Disposition: the configured ADM stage key and DDP Atmos factory/module are statically bound; the first track-count gate remains unbound.

## Bound selected route

Ghidra 12.2 evidence binds the hash-matched `dee.exe` StageRunner path for the configured input type `atmos_mezz` and filter key `encode_to_atmos_ddp`. The selected factory table entry in `dee_audio_filter_ddp_atmos.dll` resolves to `FilterDDPJocEncPlugin`; its source type list includes `audio.atmos_mezz`, and the output route advertises `audio.ec3.atmos`.

The separate `FilterConvertAtmosMezz` class in `dee_audio_filter_convert_atmos_mezz.dll` is not shown as a separate StageRunner stage selected by `encode_to_atmos_ddp`. This does not exclude internal use elsewhere; it only bounds the configured stage selection.

The RAW PCM `pcm_to_ddp` path in `dee_ddp_encoder.exe` has a max-eight-channel gate, but no typed edge binds that route to the selected `atmos_mezz` StageRunner input. Do not apply that gate to ADM without a proven internal transfer.

## Remaining boundary

The StageRunner type boundary carries a source type string, not a numeric track count. The external DLL-to-stage key is bound for the selected factory, but the first 36/38 count-bearing field inside the selected AtmosMezz/DDP implementation and the reader-to-process count transfer remain `UNBOUND` (see R3/R2 supplements). No DEE run or payload read occurred.

## Upstream package

- Package: `DEE-STAGE-FACTORY-BINDING-V1`
- MANIFEST.json SHA-256: `49ca286cf86d221d42413bd5dced050e2413651c88dcef1d453f85e3b028d8b8`
- Two payload sizes and hashes independently matched.
- Checker disposition: `PASS_FACTORY_KEY_TO_DDP_ATMOS_MODULE_WITH_COUNT_GATE_UNBOUND`.
