# ADM/BWF reader factory and first handoff boundary

## R138 source binding

The hash-bound R138 configuration selects route `encode_to_atmos_ddp`, input type `audio.adm`, and the corrected `embedded_timecode` profile. The `timecode_frame_rate` tag is `not_indicated`.

## Static reader roles

The host contains a WAVE-file input adapter with property, initialization, and run stages. Its run path consumes a reader-provider member and makes a direct call to a shared source-time metadata normalizer using timecode/offset/FFOA metadata maps. That call is a class-local static edge; the shared helper is used by multiple input adapters, and the R138 `audio.adm` type has not been joined to this WAVE adapter.

The audio storage library contains ADM/BWF file-storage, descriptor, and WAVE-storage components. Static component presence does not show that R138 selects that concrete factory or transfers its parsed descriptor into the host adapter. No typed provider/parser ownership or generation join was recovered.

## R138 boundary

The concrete reader factory selected by input type `audio.adm`, the host-provider-to-ADM-parser edge, and the first reader-output-to-3D-Atmos/codec consumer remain `UNBOUND`. The generic WAVE adapter metadata edge is not counted as the selected R138 path. The AtmosMezz-specific reader remains non-gating for this ADM profile because no explicit dispatch into it was established.

No authorized audio sample was used in this static Ghidra turn. No native run was started; Wash/R4 has the consolidated observation coordination.
