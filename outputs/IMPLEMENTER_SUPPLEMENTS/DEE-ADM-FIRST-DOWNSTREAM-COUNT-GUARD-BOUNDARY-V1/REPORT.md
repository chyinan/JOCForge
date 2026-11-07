# DEE first downstream row consumer and count guard

## Disposition

FIRST_ROW_CONSUMER_AND_BED_COUNT_GUARD_BOUND_STATIC; NO_TOTAL_N_OR_K_CAP_FOUND_IN_THIS_SLICE; PROFILE_COUNT_C_SEPARATE; FULL_CHAIN_SUPPORT_NOT_ESTABLISHED.

FUN_1801ad9d0 dispatches active source-map rows to 0x58-byte records, writing the source slot ID in the first byte. Its first direct consumer is FUN_1801acd10. That consumer iterates canonical map length M; when a map entry is object-marked, it consumes the next record by compact ordinal uVar7 and does not read or compare the stored source ID. This slice therefore shows positional correspondence, not an explicit source-ID identity check.

The first additional count guard in this consumer is for non-object/bed entries: U32 count uVar8 must not exceed 10, and each paired low byte must be <=1; overflow returns error 6. This is not a total physical ADM row count N or object count K limit.

Later FUN_1801ad360 reads an independent U32 profile/layout count C from profile object +0x08. No typed relation to N, K, or M is shown, and codec count is outside this slice.

## Input vectors and limits

The package's N=36 and N=38 vectors, plus structural N=37, use synthetic dense slots with 10 bed entries and all object entries active, so M=K+10=N for those vectors only. Slot ordinals do not bind ADM UID identity. N=37 is arithmetic only; it is not evidence of runtime support. The earlier 1..128 ingress range is not full-chain support.

## Independent Wash checks

Wash recomputed the four source-package payloads, the source manifest, all four parent manifests and their 39 listed payload files, and the current target plugin DLL hash. The current DLL matched 3d66bcec36031fd48e6565d15f05fea656642377ca4f8c98cdce1cce8b7e95d2. The source package reports a new Ghidra read-only rerun blocked by an existing project lock; this supplement uses its hash-bound existing analysis files and makes no claim of a new Ghidra run. DEE/runtime, dynamic capture, media/PCM/payload/key/license access, and implementation contact are NOT_RUN/false.

## Implementation boundary

This is one downstream consumer boundary, not an arbitrary-N acceptance certificate. Keep N, K, M, and profile count C separate. The only guard bound here is the bed/non-object-entry count; do not turn it into a 38-row or total-N limit.
