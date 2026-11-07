# Remaining MeteringMode token boundary

Reference-only Wash supplement. `1770-2`, `1770-1`, and `LeqA` are schema-listed, but their token-specific translation, selected Meter/provider, and measurement/audio effects remain unbound/not run. `IMPLEMENTATION_READY=NO` for these modes.

The reused generic consumer `FUN_1801535b0` reads a string through vtable `+0x30` at `param_1+0x210`, looks it up in `DAT_180a17e08`, and on a successful map hit writes a raw four-byte U32 candidate to `param_2+0x90`. This package captures no raw values for the three tokens. The XML descriptor member at `param_1+0x1f0` is not joined to that receiver, the selected Meter/provider, or a generation.

The XSD copies contain distinct `measure_only`, `measure_and_correct`, and `loudness_measurement` contexts and defaults. Do not merge these defaults into a runtime claim. Descriptor notes that DI is ignored for `1770-1` and `LeqA` are documentation, not observed runtime branches.