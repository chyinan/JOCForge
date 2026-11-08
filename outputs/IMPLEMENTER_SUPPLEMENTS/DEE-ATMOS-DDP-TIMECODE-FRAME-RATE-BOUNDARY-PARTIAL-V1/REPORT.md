# Atmos DDP timecode frame-rate boundary

## Scope

This supplement covers only the schema enum, a host setter candidate, and the first DDP property-map reader candidate for encode_to_atmos_ddp/timecode_frame_rate. It does not establish that the host value reaches the selected DDP stage.

## Static findings

- The DDP schema text declares the field as xs:string, default not_indicated, with ten tokens: not_indicated, 23.976, 24, 25, 29.97, 30, 48, 50, 59.94, and 60.
- The host function FUN_1400f13e0 matches the exact key timecode_frame_rate, parses it via FUN_1400705c0, and stores a value at receiver byte offset +0x188. The receiver is not joined to the selected output XML node or DDP object; the same key appears on input and output nodes.
- The DDP constructor candidate initializes a timecode_frame_rate property default at candidate offset +0x3f8. FUN_18014fd90 is a reader candidate that queries a property map at param_1+0xa0 after checking param_1+0x3f8.
- FUN_18014a600 passes its param_1 unchanged to FUN_18014fd90; its owner is not bound to dee::proc::Priv. Equal numeric offset +0x3f8 does not establish object identity, value transfer, or generation.
- A separate host validator compares the same ten strings, but its call path to the output-field setter is not bound.

## Open boundary

The first unresolved edge is the host setter's typed receiver/value transfer to the same selected DDP owner and property map read by FUN_18014fd90. Runtime XML admission, frame-rate selection, timecode conversion effect, and output behavior are NOT_RUN or UNBOUND.

## Handoff status

IMPLEMENTATION_READY=NO. The ten strings are schema text only; no runtime output-field selection or codec effect is claimed. This package contains no Ghidra query source, decompilation, media, PCM, payload, key, or license material.
