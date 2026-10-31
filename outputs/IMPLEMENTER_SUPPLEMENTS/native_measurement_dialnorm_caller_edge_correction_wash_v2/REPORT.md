# Correction: observed Atmos DDP writer arguments

## Correction

This new Wash package corrects the earlier interpretation of the WinDbg register capture. Ghidra shows the helper call shape is `(field_pointer, element_count, width)`. The captured `1` was the element count; the source field scalar was **NOT_CAPTURED**. No scalar field value is claimed here.

The runtime return site maps in Ghidra to an unnamed record member at relative byte offset `+8`, passed by an Atmos DDP metadata serialization routine. That routine is called from the codec loop. Ghidra also shows the same parent-context member location read later as a short and copied into a downstream record slot; neither use supplies a semantic field tag. The earlier static U16-to-five-bit candidate is a different field and callsite.

## Open producer edge

Ghidra's targeted path is:

`Atmos DDP codec loop → metadata serialization routine → generic bit writer`

The serializer passes a pointer to its input member with element count one and width five. The observed field value was not captured. The member has no recovered semantic tag. The immediate record-builder/transformer path did not yield a typed producer for this member. Its producer, owner, and generation remain **UNBOUND**. No connection to measurement, F06ND DRC, or F12 DialNorm is established.

## Scope

The bounded run used one authorized 38-channel ADM BWF with an Atmos DDP configuration at 1024 kbps. It stopped after the first width-five helper hit; output metadata was not checked and the temporary directory was removed without reading payload. 48 kHz output, 5.1/JOC signaling, player playback, and multi-sample E2E remain **NOT_RUN**.
