# E-AC-3 writer field classification

## Finding

Ghidra maps the corrected run11 helper hit to an Atmos DDP metadata serialization routine on the codec loop path. The same routine writes the E-AC-3 frame sync word through the bit writer before serializing its record fields. The observed width-five call therefore belongs to a bitstream serialization path, not only to measurement logging.

The helper arguments are `(field_pointer, element_count=1, width=5)`. The earlier register value `1` was the element count; the field scalar was **NOT_CAPTURED**. The runtime member is distinct from the earlier static U16-to-five-bit candidate. Its semantic tag, typed producer, owner, and generation remain **UNBOUND**. Whether this specific field affects decoded audio cannot be determined without its tag.

## Open edge

Ghidra found the serializer's direct codec-loop path and a later read of the same parent-context member, but not its constructor or first writer. The next exact edge is the typed producer for that unnamed record member. No measurement/F06ND/F12 DialNorm relation is established.

## Scope

The run11 config hash, filter/route key, input type, and factory index were not retained and are **NOT_CAPTURED**. No further native run was started. Playback and full ADM-to-JOC E2E remain **NOT_RUN**.
