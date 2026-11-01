# DDP source property typed-map lineage

## Result

For the hash-bound `dee_audio_filter_ddp.dll` image, static Ghidra evidence now distinguishes two flows in the `dee_audio_filter_source` property handler. It binds a metadata-map setter/getter path, but does not bind that map to the earlier host-source writer `param3` or to an ADM reader instance.

## Typed paths

The outer `FilterDDPEnc` state pointer is at outer `+0xb8`; its state size is `0x1590`. In `FUN_1801ed430`, the property handler uses `FUN_18020d820` with the per-filter map at state `+0x1530`. That function returns a normalized string, later stored at outer `+0xc0` and sent to an input reader's `input_file` setter. This string path is not directly passed as a map-valued argument to the metadata setter.

The same property handler separately extracts an XML child string and calls vtable slot `+0x28` on the `MediaMetadata` object at state `+0x1f0`. `FUN_18023ec60` parses a map containing fields such as `filename` and `index`. The map is committed through vtable slot `+0x50` and stored as `SafeType<vector<map<string,string>>>`; the matching getter at vtable slot `+0x30` reads the `filename` key.

These static producer/consumer relationships do not establish a runtime filename value or that the metadata map reaches the host writer. The previously observed host `param3` is from a different image hash (`3d66…`); this package targets `2fd503…`. Keep the cross-image writer join `UNBOUND`.

## Runtime boundary

The linked metadata-only capture did not capture a setter record. DEE audio consumption remains `UNKNOWN`; bitstream and playback remain `NOT_RUN`. No extra DEE run occurred. No source code, Ghidra scripts, decompilation, PCM, output payload, or key material is included in this supplement.

## Scope

Use this as a static configuration/data-model reference. It does not prove the selected ADM reader, audio read behavior, host source writer ownership, generation, or any E-AC-3/JOC bitstream field. The input-only vector is a structural data-flow discriminator, not an executed XML fixture.
