# Preferred-downmix opaque callback consumer boundary

## Disposition

OPAQUE_CALLBACK_SUBOBJECT_BRANCH_BOUND_STATIC; SELECTED_LORO_TO_CALLBACK_OR_DMIXMOD_UNBOUND; AUDIO_OR_METADATA_EFFECT_UNBOUND; RUNTIME_NOT_RUN.

This addendum follows one static callback path from an opaque descriptor value at +0x6368 to a subobject calculation and an indirect callback. It does not bind that callback to the selected XML value preferred_downmix_mode=loro or to the native dmixmod typed field.

## Bound static path

At helper address 0x14045b640, the existing Ghidra instruction stream reads owner byte +0x1e8, selects subobject offset +0x248 or +0x1f0, adds that offset to the owner, and returns the subobject address. The helper has no Ghidra function definition in the analyzed project; only its scoped instructions are used here. The flag producer and active branch remain UNBOUND.

FUN_140473290 receives the subobject and string storage, then invokes the function pointer at subobject +0x8 with the owner and string-data pointer. The callback target, initializer, owner type, and caller status semantics remain UNBOUND. Separately, the selected callback path passes callback +0x70 to FUN_1404802e0, which reaches a native dmixmod parser; no typed join connects that parser path to the opaque callback pointer or the selected loro value.

The source package reports a selected-source XML value of loro, but has no runtime XML capture. Wash did not independently reopen that XML, and this static path does not join it to the callback object. The 36/38 ADM material is metadata-only and is not used to infer audio behavior.

## Independent Wash checks

Wash recomputed the source manifest, all five payload hashes and sizes, all six SHA256SUMS entries, both Ghidra query scripts/logs, the current target executable, and the available parent manifests. The executable matched 4b4c624d94a5b47d97b62b28073e1bd351b2deeeea039a7ccfaa2af96562c8a7. One referenced historical parent directory was not located in this pass and is not used to support this boundary. The source agent reports normal and python -O checker PASS; Wash did not execute that checker.

No DEE run, WinDbg attach, A/B, media/PCM/payload/key/license read, spatial-object rendering, or implementation contact is claimed.

## Implementation boundary

Do not infer a user-selected mode, default, validation result, callback target, or audio effect from this helper path. The next minimum query is the initializer for the object stored at descriptor +0x6368, focusing on flag +0x1e8 and callback pointer at the active subobject +0x1f8 or +0x250.
