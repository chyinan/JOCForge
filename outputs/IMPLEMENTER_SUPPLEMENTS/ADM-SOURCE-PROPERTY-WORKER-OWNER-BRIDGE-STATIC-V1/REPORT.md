# ADM source property to worker owner bridge

## Static owner chain

For the identified `audio.atmos_mezz` → `encode_to_atmos_ddp` StageRunner route, the factory create slot returns a processing stage object `P2`; the host stores that exact return pointer at `H+0x270`. `RunAudioFilter` passes `H+0x448` under property key `dee_audio_filter_source` through the generic host property packet to `P2` virtual slot `+0x08`.

The P2 property entry loads private state `V` from `P2+0xB8`, forwards the packet to the shared handler, which resolves the key in V's parameter registry and calls the resolved string setter on parameter owner `Q`. The selected `dee_audio_filter_source` schema parameter Q prepares/validates and commits its value into Q-owned storage on success. The P2 lifecycle execution slot `+0x28` independently reloads the same `V` from `P2+0xB8` and passes it into stage execution. This statically closes the typed host-source-property → P2 → V → Q edge for the identified class; the actual selected instance/value/generation was not captured.

The inherited source-property evidence connects Q's getter through the workflow source vector to worker configuration source vector `S`. Under the inherited ordinary-reader/PE branch, S supplies the source reference to the storage factory, where the previously washed K→J→B→D count lineage applies. These links are conditional on the identified branch and do not prove a runtime-selected source instance.

## Input-only vectors

- With key `dee_audio_filter_source` and a valid ADM source reference, the static rule routes the value through P2's generic property packet to Q's setter, then the same V is used at stage execution. This is a type/owner vector, not an observed run.
- On the inherited ordinary-reader/PE branch, a count N in S can follow K→J→B→D; prior count vectors N=36 and N=38 are representable by that static chain. Runtime count values, selected branch, and acceptance are `NOT_RUN`.

## Remaining boundary

The selected P2 token and source generation, actual property value, encoding pass index, reader/PE branch values, and native count/frame values remain unobserved. A pointer-only runtime receipt may bind those fields without dereferencing sample buffers. Do not infer selected identity from equal counts, values, or event order.

## Provenance

- Upstream package `ADM-SOURCE-PROPERTY-WORKER-OWNER-BRIDGE-STATIC-V1`, manifest SHA-256 `a4f09e46efa60550322478d5f90d4a29935c087aee3231ac000ac410a9ade945`.
- Wash matched all 6 package files, all 12 research receipts, and all 7 prior Wash manifest links.
- Ghidra 12.2 read-only static analysis. Native execution, payload reads, and private key/license access are `NOT_RUN`/`NOT_READ`.
- No Ghidra logs, scripts, decompilation, source code, PCM, or media are included here.
