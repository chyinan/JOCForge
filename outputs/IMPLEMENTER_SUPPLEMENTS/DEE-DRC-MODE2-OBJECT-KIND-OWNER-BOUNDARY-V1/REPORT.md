# Mode-2 output object-kind owner boundary

## Disposition

MODE2_INNER_CONTEXT_OWNER_AND_KIND_WRITER_BOUND_STATIC; SELECTED_KIND_VALUE_AND_SLOT_TARGET_UNBOUND; LINE_A0_AND_EFFECT_UNBOUND; RUNTIME_NOT_RUN.

The selected mode-2 setup allocates per-item objects in an inner context. FUN_1801b0750 writes each object pointer to inner-context +0xc0f8 + index*8 and writes the object's first dword from inner-context +0xc13c. FUN_1801ab800 later reads the wrapper at context +0x810 and passes the same inner-context pointer to FUN_1801b1590. The writer and reader therefore share a statically bound inner-context owner.

FUN_1801c3150 prepares +0xc13c from a paired value for configuration selector 1 or its static-default branch. The selected runtime kind is not captured, and the semantic relationship to line_mode_drc_profile or record +0xa0 is not bound.

FUN_1801b1590 indexes DAT_1806521e8 by the object's first dword times 0x30. The +0x28 callsite is 0x1801b18d9, but its selected row target is unresolved. Rows 0..7 were sampled as zero; this is not a whole-table absence claim. No first typed BSI/profile read or audio effect is established.

## Independent Wash checks

Wash recomputed the source evidence manifest and package seal, all seven sealed files, eight source evidence hashes, the parent manifest/seal pair, and the current target plugin DLL hash. The source agent reports normal and python -O checker PASS; Wash did not execute that checker.

No DEE run, runtime selection, media/payload/key/license read, or implementation contact occurred.

## Implementation boundary

This closes the mode-2 inner-context owner and kind writer only. The actual kind value, selected table slot +0x28 target, line +0xa0 relation, and final BSI/audio effect remain UNBOUND.
