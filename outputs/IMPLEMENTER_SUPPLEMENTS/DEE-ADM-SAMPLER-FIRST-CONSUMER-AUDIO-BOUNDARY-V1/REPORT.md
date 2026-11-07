# Sampler state first-consumer boundary

## Disposition

NESTED_SAMPLER_STATE_READ_MODIFY_WRITE_BOUND_STATIC; CALLER_OUTPARAM_POINTER_BOUND; POSTCALL_OUTPARAM_CONSUMER_UNBOUND; PCM_AUDIO_OWNER_UNBOUND; RUNTIME_NOT_RUN.

FUN_1801b6fa0 first reads the same nested receiver's +0x8c/+0x94 into local state before the conditional recurrence. After FUN_1801caa90 returns, it writes the resulting state back to those same receiver fields. This is a same-object sampler-state read/modify/write; the semantic audio role and any PCM owner remain UNBOUND.

At the examined FUN_1801ad9d0 callsite, RCX is the current per-slot receiver and RDX points to the caller's stack local at RBP-0x80 (reported as local_d8). FUN_1801b6fa0 writes a descriptor snapshot through param_2. The current source traces do not show what FUN_1801ad9d0 subsequently does with that local, so propagation into the row descriptor or audio block is not independently bound here.

The +0x80 branch remains conditional; no runtime value was used to decide it. No PCM/audio-block pointer appears on the verified state read/write edge.

## Independent Wash checks

Wash recomputed all four source payloads and lengths, the five SHA256SUMS entries, manifest and seal bindings, all 17 source-manifest entries, both parent manifest hashes, and the current target DLL hash. The target DLL matched 3d66bcec36031fd48e6565d15f05fea656642377ca4f8c98cdce1cce8b7e95d2. The source agent reports normal and python -O checker PASS; Wash did not execute that checker.

No DEE run, runtime debugger/capture, PCM/media/payload/AU/EC3/key/license read, or implementation contact occurred.

## Implementation boundary

The sampler fields are not established as PCM samples or an audio buffer. This package adds no DSP rule and does not explain any PCM mismatch.
