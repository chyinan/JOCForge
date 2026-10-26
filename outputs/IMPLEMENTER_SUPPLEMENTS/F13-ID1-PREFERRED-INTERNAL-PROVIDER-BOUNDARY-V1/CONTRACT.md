# ID1 preferred internal measurement provider boundary

Status: `INTERNAL_PROVIDER_AND_BOOLEANIZATION_STATIC_BOUND; TAG_SEMANTICS_SELECTED_VALUE_AND_RUNTIME_EPOCH_UNBOUND`

`preferred` is not a task XML/UI input. The approved F13.6 consumer requires an internal boolean; the static evidence traces that bit to an existing measurement-result I32 member. Do not ask the user to choose it, add a task field, or infer it from LoRo/LtRt, `preferred_downmix_mode`, or `gating_kind`.

## Types and value rules

- The measurement-result member is an opaque dimensionless I32 tag copied as an integer. It is not derived by converting the F32 loudness values.
- ID1 `config[2]` is the canonical I32 boolean `1` exactly when the copied source I32 equals `1`; every other I32 produces `0`.
- Other F32 measurement members rank candidate result records. The whole 44-byte record is replaced only when a candidate loudness is strictly greater than the current comparator. Equal loudness keeps the earlier record.
- The aggregate record starts as 44 zero bytes; its loudness comparator starts at F32 `-120`. If no candidate wins, the member remains zero as a local initialized-record fallback. This is not a missing-task-field default and does not establish the selected job's value.

## Existing provider paths

The provider factory selects one of two internal measurement objects from existing routing fields; the selected branch for the target job is unobserved.

1. The prepare-stage path uses a zero-size processing call to flush measurement state, then copies the member from the finalized info record into a cache. Reading before the finalized flag is set fails; it does not fabricate a value.
2. The processor-backed path reads an internal I32 tag and remaps `3 -> 2`, `2 -> 1`, `1 -> 0`, all other values to `-1`. That result occupies the same measurement-result member. The tag values remain unnamed; no gating or downmix enum alias is established.

The saved result record flows through plugin-owned storage and worker configuration into encoder-owned private/open configuration. Encoder-open applies the strict I32 equality rule to produce ID1 `config[2]`; the reviewed per-source wrapper copies the nine-I32 template to the timer/writer input on its conditional branch. This is internal configuration state, not a vendor key. Preference construction is not derived from AU index, queue storage ID, or timer expiry.

## Remaining limits

Still unbound: the semantic names for provider tags; the approved-model `MeasurementReceipt` crosswalk; which provider branch, source tag, and resulting bit the selected job uses; and the same-programme/source/reset epoch. The direct per-source wrapper is conditional and its selected-run reachability was not observed. A model caller copy was not available in the bounded readable scope; no global absence claim is made.

The static analysis was performed from a cached Ghidra database whose import metadata names the expected image SHA. Nineteen recovered code-body hashes and the package's static verifier were checked, but the original PE could not be freshly rehashed: the exact DLL was not present under the supplied `D:\Dolby_Encoding_Engine` root. Therefore this is cached static evidence, not a fresh binary identity check or runtime observation. No native execution, model execution, measurement value, PCM, or key is included.

Use the local rules and input-only controls in this supplement. Keep the unresolved selected branch/value and epoch explicit; do not block unrelated implementation work on a user-provided choice or a guessed tag name.
