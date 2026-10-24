# Selected production cold-root observations

This internal supplement records an actual selected wrapper/profile top-constructor observation plus independent cold queue-control reconstruction. It does not establish whole production cold-parent equivalence or end-to-end rollback.

## What the captures establish

The selected run entered the actual top codec constructor, not the previously found compatibility branch. Its anonymous top/parent identity is bound across the constructor, frontend cold initialization and six parent/frontend invocations. The observed gate advanced by 1536-sample steps; caller-visible bytes were 0, 0, then 4096-byte AUs. Six naturally observed assembly results were 221, 221, 0, 0, 0, 0; parent tolerance began at 2 and reached 0 without a fatal return. These are natural priming observations, not injected failure results.

R20 separately reconstructs five queue initializers in fully owned scratch arenas. It checks descriptor-count/phase/read-index/empty-slot rules, pointer-table ownership, guards and non-aliasing. This is a queue control initializer only; it is not byte-exact equivalence for the full codec owner.

## Capacity field caveat

R20 records the native constructor field named `capacity_bytes` as ID1=10, generic ID6=3, ID11=2103, ID14=543 and ID2=8, with delays/periods/descriptors in `CONSTRUCTION-GRAPH.json`. The ID11 value 2103 has not yet been tied to the approved F13.4 per-payload limit of 160 bytes; treat it only as the observed constructor field. Do not use 2103 as an ID11 payload maximum or replace the approved 160-byte contract until Wash resolves the field's actual consumer and semantics. Generic ID6 is distinct from ID2.

## Failure and publication limits

The package contains static failure-root behavior, but no production failure injection was run. Durable file commit is NOT_OBSERVED. Caller-visible frontend gate bytes are not a durable transaction receipt. The R19 scratch-parent provider/container cases have not been connected to these production cold roots.

The actual serializer constructor installs a built-in protection-provider identity; its contents were not read. R20 did not read the configuration block or private key, modify live roots/parameters/branches, or load raw PCM. It does not supply a way to reproduce the production protection identity with an independent graph.

Required materials remain: selected top-constructor configuration/admission schema; an approved nonsecret selected protection-provider identity contract; complete codec cold-root initializer/failure snapshot schema; and a durable publication transaction fixture. R20 records the intended purpose and acquisition condition for each.

The R20 replay is limited to queue cold rules, frontend gate, tolerance and owner joins. Whole F13C, production-cold parent equivalence, full-parent failure rollback and product acceptance remain NOT_READY.
