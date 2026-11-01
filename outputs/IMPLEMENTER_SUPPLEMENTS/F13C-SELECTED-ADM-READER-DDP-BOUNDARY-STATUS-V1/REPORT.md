# Selected ADM reader to DDP row boundary

## Static finding

The scoped Ghidra trace found one direct `audio.adm` string reference in the host image. It belongs to `FUN_140035700`, which constructs an output-side `AtmosMezzPlugin` supported-type list. It does not select the R138 `audio_input` reader or factory. Generic audio-input registry code and DACT ADM/BWF components exist, but no typed DACT getter-to-host-registry or R138 owner-to-selected-factory edge was recovered. The route-profile lookup and audio-filter index enumeration also have no same-owner transfer in the traced paths.

The selector-3 opener-to-cloned setup `param4[2]` copy chain is statically established by prior washed evidence; the source and clone are different allocations. The setup read adapter is a member of the same class vtable, but its runtime instance and generation are not joined. A prior single native capture binds the DDP packet row table through its plugin copy to the `FUN_1801AC670` row pointer/count. That capture is not linked back to the selected ADM reader.

## Observation plan and limits

The incoming package supplies a one-activation, metadata-only capture plan covering selected audio-input factory/reader, opener table, clone pointers, setup `param4[2]` and owned table, converter read adapter, DDP worker/map, and packet/normalizer row pointers and counts. It is a specification only; this package contains no result from that observation. Join only by same activation, typed owner pointer, call-stack identity and an explicit parent/task generation token when present. If generation cannot be captured, preserve it as `UNBOUND`. Stop when a path is not reached.

No new native run, PCM/payload/key access, or bitstream emission occurred. Absolute selected-reader-to-DDP-row identity remains `UNBOUND`.
