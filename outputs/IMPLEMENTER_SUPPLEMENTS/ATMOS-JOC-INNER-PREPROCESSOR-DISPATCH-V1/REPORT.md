# Atmos JOC inner preprocessor selector dispatch

## Classified branch

The nested call from `SinglethreadSymphonyRenderer` to the component at renderer `+0xe0` goes through `vtable+0x28`, but that slot is not a sample transform in the resolved zero-selector branch. The config selector is returned by `param5.vtable+0x38` in `FUN_180435120`:

- Selector `0`: constructs `Dolby::Atmos::SpatialCoderPlatform`. Its `+0x28` path is a state/control update (`FUN_180443920 → FUN_1804493d0 → FUN_1803bc720`) with no sample-buffer operation. The actual spatial coding method is a separate slot: `+0x08 → FUN_1804437d0 → FUN_1804497c0`, which contains the `sc_process` path and float-backed work buffers.
- Nonzero selector: constructs `Dolby::Atmos::Home::PreProcessorBypass`. Its `+0x28` entry is CFG-guarded; the concrete indirect target remains `UNBOUND`.

The best-supported implementation rule is conditional: preserve the typed config selector; for zero, run the SpatialCoder sample-process slot `+0x08`; for nonzero, preserve the bypass branch and its unresolved callback. Do not implement the generic `+0x28` call as QMF/audio processing.

## Confidence and limits

Confidence is `0.90` for the static branch targets and their roles. The selector value for any particular ADM run remains `UNBOUND`. Two prior authorized receipts cover 36-channel, 48 kHz `audio.adm → encode_to_atmos_ddp` inputs, one active and one quiet; this slice started no DEE run. The authorized `人是猫` 38-channel runtime branch was not observed. The bypass callback target and exact downstream audio equivalence remain open; neither blocks implementing the conditional dispatch contract.

## Input-only vector and exclusions

The vector changes only the selector while holding 36 row tokens and symbolic frame count constant; it was not executed and contains no PCM. No source code, DEE binary, disassembly, decompilation, audio, PCM, key, or license material is included.
