# LFE aligned-helper scalar first-use boundary

## Bound static edge

This supplement follows the selected aligned helper return in `FUN_140340640` from `FUN_1403174c0`. The returned scalar is stored locally and transformed into an integer value passed in `R9D` to `FUN_14036bc60`. In the inspected target body, the first operation that mentions `R9D` replaces it with a value derived from another register before any read of the incoming argument. After the call, the caller stores the returned `XMM0` value to the output lane; it does not reload the scalar local or use the scalar-derived registers on that path.

The narrow conclusion is that the aligned-helper scalar does not flow through this call's `R9D` argument into the post-call `XMM0` store in this inspected path. This is a bounded static dataflow result, not a global no-effect claim.

## What remains open

Broader buffer and state effects, runtime owner/generation, numeric parity, and any causal connection to a measured LFE difference remain `UNBOUND` or `NOT_RUN`. No DEE run, PCM/media/payload/key/license read, or implementation source inspection was performed. The user-corrected recurrence selector finding remains separate: the two inspected core dispatchers set the LFE recurrence selector to zero.

## Provenance

Source package `dee_lfe_reduction_scalar_first_use_wash_v1` has evidence-manifest SHA-256 `df13b0e319af4aca755a01d5d50505e23ff4c7afa37ea9af5cb542a19be5499c` and package-seal SHA-256 `5a1aea71baf92d6835f4342631474d113bf8aa5caa4441c879545126767d4066`. It reuses Ghidra 12.2 PUBLIC read-only/no-analysis Listing evidence for `FUN_140340640` and `FUN_14036bc60`; source identities and parent receipt hashes are recorded in `SOURCE-MANIFESTS.json`.
