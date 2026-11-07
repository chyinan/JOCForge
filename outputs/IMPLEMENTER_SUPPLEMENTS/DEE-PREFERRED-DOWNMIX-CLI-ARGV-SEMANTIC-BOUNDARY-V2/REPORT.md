# Preferred-downmix CLI argv and typed-storage boundary, V2

## Corrected bootstrap path

The earlier V1 stopped too early at the app-entry call. In the newly hash-bound Ghidra slice, `FUN_1400b5310` dispatches vtable slot `+0x20` to `FUN_1401da950`; this target receives `this`, `argc`, and the converted `argv` pointer in `R8`. `FUN_1401da950` calls `FUN_1401d5ad0`, which builds a string vector from `argv[1..argc)` and passes it in `RDX` to `FUN_14029d4d0`. The inspected ChildOp target initializes its object but does not read that `RDX` argument; actual argv-content consumption is not established.

## Parser and commit path

The configured ChildOp path reaches `FUN_14029fe70`. Its option-semantic vtable calls at `+0x08` and `+0x10` return token-count bounds. Parsed option records are copied into an application option-record collection and merged into the configuration tree at `this+0x18`. That tree merge is distinct from the Boost string storage at `DdpjocEncoderApp+0x360`.

The traced path does not establish the selected semantic object as the typed `encoder` value and does not show a `parse`/`store`/`notify` write to `this+0x360`. The earlier V1 first-break at the app-entry vtable `+0x08` is superseded by the deeper V2 edges; the exact remaining boundary is ChildOp argument consumption and any later typed-storage writer.

## Limits

Actual child argv and runtime selection were not captured. The static trace does not prove Boost storage is absent elsewhere. No DEE run, media/PCM/payload/key/license read, or implementation-source access was performed.

## Provenance

The source receipt `preferred_downmix_encoder_boost_semantic_boundary_wash_v1` has manifest SHA-256 `064b940f13846be0fe512c061d4ef8472311cbbbf90da707d39e574d165c8ed8` and passed its normal and `python -O` checker. This V2 refines Wash supplement V1 (`a168d53ca33fba23029b02bfa319188ccea84895e6ee597f2bd9859f7d135007`, seal `eff2780bb1bb7e8d6664493ac06f91a8f31871143a44d8db8aa5cb6845a144fe`).
