# Wash review: same-owner E-AC-3 metadata field path

Evidence-only supplement from a Ghidra static package.

- One parent path passes the same owner object to a setter and later to the serializer. On this path, the setter stores U16 value 16 at owner-relative byte offset 102848; both analyzed branches store that value.
- The serializer receives a base at owner-relative byte offset 102840 and sends its +8 member to a generic writer with element count 1 and width 5. This is the same owner-relative field.
- The field's E-AC-3 semantic tag remains UNBOUND. It is NOT_ESTABLISHED to be DialNorm. Runtime value capture, lifetime first writer, object type, measurement/DRC relation, playback impact, full E2E, and playback remain NOT_CAPTURED/UNBOUND/NOT_RUN as specified in EVIDENCE.json.

Do not infer a codec field or implement value 16 from this static layout alone. This supplement closes a bounded writer lineage edge, not field meaning or end-to-end E-AC-3 JOC behavior.
