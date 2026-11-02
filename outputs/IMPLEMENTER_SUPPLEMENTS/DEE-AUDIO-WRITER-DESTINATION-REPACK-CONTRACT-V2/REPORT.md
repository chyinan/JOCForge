# DEE writer, repack, and stream-object handoff

The prior supplement bound the residual U16 flush and the `FUN_18008d260`
repack destination. This updated static trace adds the next ordered handoff:
`FUN_1800b7930` iterates per-unit records in ordinal order, passes each
repacked `+0xb90` buffer and `+0xb9a*2` byte length to `FUN_180077ac0`, which
selects a per-index stream object and calls `FUN_1800b53b0` with that object
and the buffer descriptor.

The observed continuation invokes internal processing and an SHA-256 state
update. It does not reach a file/container write in the traced chain. Thus
writer-to-repack-to-consumer identity and unit order are bound, while the
application file writer remains outside this slice. This is useful for the
bitstream/assembly implementation but is not a product file parse or playback
result.

No DLL runtime, media, PCM, encoded payload, key, or license was accessed; no
Ghidra scripts or binary are included in this Wash supplement.
