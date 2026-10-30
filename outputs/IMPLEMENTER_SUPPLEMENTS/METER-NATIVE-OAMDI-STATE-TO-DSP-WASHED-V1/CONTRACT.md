# Native OAMDI state scalar to DSP context

## Bounded run-032 edge

For the selected native profile/object path, static field flow shows the property update reading a signed I32 at state offset `+0x18`, passing it through a helper as the DSP function's fifth argument, and the DSP function consuming that argument into its context by storing its negative. In run 032, a one-shot observer joined the state field to the fifth argument via the direct call stack returning into the property update function. The state owner token is the same native OAMDI owner `S`.

The join is established by static field flow plus direct call-stack scope, not by comparing values. Both observed scalar values were zero, but equality is not the proof. The field's semantic name remains `UNBOUND`. The separate `+0x28` pointer passed to the DSP function is not referenced by the analyzed DSP body.

## Limits

No native callback generation token or F06OF model generation join was captured. This does not bind the scalar to downmix index, carrier block, control16 row/word, or a named property. The run's EC-3 output is hash-only; this package does not prove the scalar changed the output. The native DSP/property output to final EC-3/BSI serializer or queue remains open. No PCM, table/control row payload, or encoded output bytes were read.
