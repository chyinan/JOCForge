# Wash review: E-AC-3 writer field classification

Evidence-only supplement. The source package manifest is hash-bound in MANIFEST.json.

- Static Ghidra evidence places the corrected width-5 helper call on an E-AC-3 bitstream metadata serializer path.
- The helper arguments are field pointer, element count 1, width 5. The prior register value 1 is the element count; the field scalar was NOT_CAPTURED.
- The unnamed member's semantic tag, typed producer/first writer, owner, and generation remain UNBOUND.
- It is NOT_ESTABLISHED that this member is DialNorm. Measurement-to-F06ND and F06ND-to-F12 joins remain UNBOUND.
- Run configuration, input type, route, and factory index were NOT_CAPTURED. Playback and end-to-end execution are NOT_RUN.

This evidence corrects the classification of the candidate path; it does not provide a field value or an implementation rule. Do not infer a tag, scalar, measurement relationship, or audible effect from this package. It is not an implementation-ready algorithm specification.
