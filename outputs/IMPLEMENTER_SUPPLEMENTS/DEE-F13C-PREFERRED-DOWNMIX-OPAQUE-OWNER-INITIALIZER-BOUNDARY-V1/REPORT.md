# Preferred-downmix opaque-owner initializer boundary

## Disposition

OPAQUE_OWNER_BRANCH_AND_INDIRECT_CALLBACK_BOUND_STATIC; FLAG_AND_CALLBACK_WRITERS_UNBOUND; SELECTED_LORO_JOIN_UNBOUND; EFFECT_UNBOUND.

The opaque object at descriptor +0x6368 reaches helper address 0x14045b640. The existing Ghidra instruction stream reads owner byte +0x1e8, selects subobject +0x248 or +0x1f0, and returns that subobject address. FUN_140473290 then invokes the function pointer at subobject +0x8 with an owner and string-data pointer.

No constructor or registration writer is bound for the +0x1e8 flag or the active callback pointer at +0x1f8/+0x250. The callback target, owner type, and relationship to preferred_downmix_mode=loro or native dmixmod remain UNBOUND. The helper does not establish a typed DDP downmix consumer or an audio effect.

## Independent Wash checks

Wash recomputed the source manifest, all five listed payloads, all six SHA256SUMS entries, both query script/log pairs, all three referenced parent manifests, and the current dee_ddpjoc_encoder.exe hash. The executable matched 4b4c624d94a5b47d97b62b28073e1bd351b2deeeea039a7ccfaa2af96562c8a7. The source agent reports normal and python -O checker PASS; Wash did not execute it.

The source package's selected XML value loro is not a runtime capture and was not independently reopened by Wash. The 36/38 ADM range is not used to infer this callback's behavior. No DEE run, WinDbg attach, A/B, media/PCM/payload/key/license read, spatial-object rendering, or implementation contact is claimed.

## Implementation boundary

Keep this callback as opaque. Do not treat its flag, subobject choice, or string callback as the selected downmix mode until a typed initializer and consumer are bound. The next minimum edge is the constructor receiving Factory<AtmosMasterFileStorage> +0x8 and writing +0x1e8 plus the active callback slot.
