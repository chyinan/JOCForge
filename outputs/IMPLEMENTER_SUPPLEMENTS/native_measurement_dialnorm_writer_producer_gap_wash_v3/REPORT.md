# Serializer field producer boundary

## Finding

Ghidra mapped the run11 width-five helper call to a member pointer inside an Atmos DDP metadata serialization routine on the codec loop path. The helper call shape is `(field_pointer, element_count=1, width=5)`. The prior register value `1` was the count; the member scalar was **NOT_CAPTURED**.

The pointer targets an unnamed record member at relative byte offset `+8`. In the same parent context, Ghidra shows that member location being read later as a short and copied into a downstream record slot. This establishes a static same-context use; it does not recover the field's semantic tag, constructor, first writer, typed producer, owner, or generation.

The earlier static U16-to-five-bit candidate is a different field and callsite. No relation to measurement, F06ND DRC, or F12 DialNorm is established.

## Targeted search boundary

The metadata serializer has one direct caller in its parent codec stage, which is called from the codec loop. The parent passes the same record base through three immediate builder/transformer helpers before serialization. A targeted Ghidra inspection did not identify a typed producer for the member in those helpers. The next unresolved edge is the constructor/initializer or first writer of this record member.

## Status

The retained run11 launch metadata gives the input argument filename `selected_adm_38ch.wav`, but the run11 XML/config hash, exact input type, filter/route key, data rate, and factory index were not retained. Do not infer them from the earlier run100 configuration or summary. The process stopped after the first matching helper call; temporary output was removed without reading payload. 48 kHz output, 5.1/JOC signaling, player playback, and multi-sample E2E remain **NOT_RUN**. The proposed two-byte field read is pending Wash coordination; no additional native run was started.
