# ID14 count and parameter-13 first-writer addendum

Ghidra static tracing now closes the selected native ID14 count constructor path. Two U32 fields in the input root are subtracted into a configuration count; the constructor accepts counts 1 through 15, stores the value in nested JOC state, and the ID14 writer emits count-minus-one as U6.

Parameter-13 is a separate property-list input. It enters configuration member +0x70 (default 11), is copied into the child worker, and reaches the additional-BSI U8. The two values are statically in the same child worker but have independent sources; no same-parent/generation observation joins them.

The first writer/owner of the two input-root fields remains UNBOUND. A second function family writes the same numeric offset but has no typed edge to this selected worker. The proposed native capture was not run because the available R138 receipt does not demonstrate a keyless profile and the prior setup stopped before input processing.

This addendum supplies a concrete producer-to-wire edge and leaves the owner/generation link open. It contains no source, decompilation, binary, PCM, payload, or private key.
