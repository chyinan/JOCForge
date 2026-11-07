# Preferred-downmix ordinal to MixMode setter boundary

## Bound typed setter call

The XML parser's mapped 32-bit code `0..3` reaches the MixMode vtable setter slot `+0x50`, resolved to `FUN_18014f6e0`. The typed destination is `dee::Types::SafeType<MixModeValue, MixMode>` in the selected `FilterDDPJocEncPlugin` subobject at `+0x910` / outer object `+0x930`.

The setter marks `MixMode+0x08`, calls `FUN_180143cf0`, and writes the helper return to `MixMode+0x50`. In the inspected XML constructor the optional callback is null; the helper returns the pre-existing `+0x50` code. Therefore the callsite's mapped ordinal reaches the setter but is not established as the value stored at `+0x50` on this constructor path.

## First break

The stored code's relation to the parsed ordinal remains `UNBOUND`; the `MixMode+0x50` audio/BSI consumer also remains `UNBOUND`. The separate `Priv+0x210` StringType member has no typed join to this MixMode wrapper. This does not establish an effective Lo/Ro or Lt/Rt output branch.

## Provenance

Source package `DEE-ATMOS-DDP-PREFERRED-DOWNMIX-TYPED-WRITE-DESTINATION-BOUNDARY-WASH-V1` manifest SHA-256 `70b680374a68e72184c6a3e3753d00533cebe4221c3b3d4b98a3180365554a76`; normal and `python -O` checkers passed. No new Ghidra query, DEE/XML runtime, media/PCM/payload/key/license access, or implementation read occurred.
