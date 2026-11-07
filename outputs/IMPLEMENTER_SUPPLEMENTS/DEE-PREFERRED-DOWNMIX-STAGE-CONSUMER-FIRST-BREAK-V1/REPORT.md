# Preferred downmix typed-reader to stage-consumer boundary

## Bound static call path

The typed MixMode getter has one direct caller in the inspected exact-entry xrefs: its formatter. No direct code caller to that formatter entry was found in the same bounded xref query. The selected setup and encode-pass functions have no direct getter/formatter reference.

The encode pass reaches FUN_1801535b0 as its first per-source candidate. That candidate is also called from another function, and its first indirect virtual call uses receiver param_1+0x210 at slot +0x30. The indirect target and any typed owner join from that receiver to the MixMode object remain UNBOUND. Equal slot offsets do not establish object identity.

## First break

The typed getter/formatter chain is not statically joined to the selected processing path. No typed audio/BSI consumer is reached. Preferred-downmix audio and BSI effects remain UNBOUND; this package makes no sound or algorithm claim.

The upstream package manifest SHA-256 is e66c7c13900ddec00aa0f5353e75dbe1ec9d2a648b6216252a3193e730c9318b; the seven-payload/eight-checksum checker passed normally and under python -O. The installed DLL was not rehashed. Raw Ghidra traces/scripts and all media/payload/key material are excluded.
