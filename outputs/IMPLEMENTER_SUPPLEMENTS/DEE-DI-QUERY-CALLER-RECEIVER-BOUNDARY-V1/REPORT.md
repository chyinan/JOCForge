# Dialogue Intelligence query-caller receiver boundary

## Bound static receiver

The upstream Ghidra receipt follows one selected call to the ParametersContainer getter. Its thunk loads the Priv pointer from FilterDDPJocEnc+0xb8 and tail-branches into the dispatcher without changing RDX. The getter receiver is therefore the same static Priv+0xec0 container whose constructor-built candidate list contains the exact dialogue_intelligence Bool.

## First break

RDX is supplied by the virtual caller and carries the dynamic query string. The static caller path does not bind that string, its input record owner, or its generation to loudness:measure_only:dialogue_intelligence. The prior exact-key-to-Bool contract applies only if that exact key is supplied. Runtime key selection and any measurement/audio/JOC effect remain UNBOUND/NOT_CAPTURED.

The upstream manifest-list SHA-256 is b39697204309881127d896f8d112523a4b2aeaa8889d92610a8d5ae968df0bec and its package-seal file SHA-256 is 4754c7ae968e1d168631e7619902de180f1d99f7a891c9019c058c3304968dbd. Its 13-file checker passed normally and under python -O. No Ghidra raw material or media/payload/key data is included.
