# Active coefficient matrix and canonical target-slot storage

Ready scope: representation adapters and current-callback matrix/destination binding. Full spatial-control producer wiring is a separate scope.

The actual frozen outer producer returns compact C[n][48], where 0<=n<=15. The matching ordered destination list d has length n and unique target slots 0..14. Production enumerates installation-active slots in ascending slot order before marker quiet. C row k belongs to target slot d[k], not target slot k. The meter worker passes this produced pair to its component caller; it does not read a cold matrix placeholder.

## Approved clean canonical adapter

Use a target-slot-indexed dense view D[15][48] for the existing 720-word canonical matrix state. Start its inactive cells at positive-zero F32 bits. Scatter by exact word copy: D[d[k],s]=C[k,s]. Gather with the same d: C'[k,s]=D[d[k],s]. This performs no arithmetic and preserves every active coefficient bit. Inactive D rows must stay positive zero. Active all-zero rows remain active because d, not coefficient nonzero tests, defines the active domain.

This explicit scatter/gather decision reconciles the approved F03W dense canonical record with the F03S/F04A compact numeric interface. It is a clean representation adapter derived from their behavior; it is not an observation that native buffers alias.

The legacy fixed-capacity view P is different: append zero rows after C and use the paired full permutation d followed by ascending inactive slots. Do not call P target-slot dense storage. Compact C/d, canonical D/identity, and packed P/full permutation preserve the same ordered per-output source/coefficient terms in the stateless F04A matrix mixer. A component consumer requiring compact rows must gather D using d. Never sort destinations without applying the same paired row operation.

Identity0..14 and the full legacy permutation are stateless mixer transport maps only. Keep the real active_target_indices=d separately for the solver, markers and stateful owner decisions. Passing a padded permutation is not permission to mark all fifteen targets active or change stateful update order.

## Callback and readiness boundaries

Capture C, d and producer generation together before marker quiet. Quiet-after flags update next target state; they cannot change this callback's matrix row count or mapping. Empty selected IDs can preserve previously active installed targets, so selected_count=0 does not imply n=0.

A legitimately executed n=0 producer returns empty C/d and a zero dense view. That is a produced result; a cold zero placeholder is only initialization. Readiness must come from execution/validation of this callback's producer, never from whether matrix cells are zero. n=0 or zero coefficients do not skip the component overlap, ring, filter or history advancement required by the existing component contract.

Validate row count 48*n, d length n, range and uniqueness before adapting. Production d is ascending installation-slot order. The adapter may preserve an explicitly paired unique order at a primitive boundary but must not broaden the producer's ordering contract.

Source identities, seven synthetic coefficient/label recipes, and three malformed inputs are hash-bound. Wash independently checked roundtrip, inactive positive-zero cells and ordered contribution-term equality. No solver, numeric audio mixer, PCM, native run or full callback was executed by these metadata checks.
