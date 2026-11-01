# F05/F13 diagnostic observability review

In the sealed Rust snapshot, ProducerTransactionMismatch is a unit error. The comparison at programme_state.rs:275 checks ID14 provenance against the supplied producer transaction, but the enum and CLI formatter do not expose expected/actual transaction, owner, phase or generation values. The runtime receipt for attempt-03 separately confirms exit 2 and keeps those values UNBOUND.

The five Rust files listed in SOURCE-BINDING.json were rehashed and match the current worktree at review time. The runner script hash in that review was later changed; this package makes no claim about the current runner instrumentation or attempt-04 receipt. The failed attempt's executable/source commit relationship remains unbound.

A minimal structured diagnostic shape is listed in EVIDENCE.json. It is a review recommendation, not an existing product API or approved audio algorithm. The synthetic cases distinguish a valid nested source/parent transaction, same-generation label mismatch, stale generation, missing provenance, and parent/source token collapse. Do not weaken transaction validation based on the old enum alone.
