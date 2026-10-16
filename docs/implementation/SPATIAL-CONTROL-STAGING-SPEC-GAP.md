# Spatial control staging specification gap

## Status

The user explicitly approved the narrower P1.3-D1 storage/staging scope in the current task handoff on 2026-10-16. This gap remains open for the future `SourceFlagsPacket` producer and keeps P1.3-D blocked; it does not block the approved D1 cold-root and staging-foundation slice. Neither packet producer may be implemented by D1.

Coordination has since authorized a separate SourceFlags producer supplement. Its manifest and files are deliberately not part of this D1 slice and will be identity-checked only after the D1 storage/staging result is committed and reported. The producer itself, `CurrentFeaturesPacket`, upstream controls/route/writer producers, and the parent transaction coordinator remain outside D1 and are not claimed as run or verified here.

## Evidence

The approved `flags_before_history` case in `crates/jocforge-domain/tests/fixtures/spatial_control_state_v5/INPUT-ONLY-VECTORS.json` provides:

- `same_before_history: true`;
- `different_current_input_or_writers: true`;
- expected relation `flags_may_differ: true`;
- expected relation `next_history_read: false`.

It does not provide concrete callback input/control/route/writer-set records, the rule producing `flags: U8[48][2]`, or concrete expected flag bytes. The v5 contract says the packet is calculated from before history and same-callback input/control/route/prefix writers, but not how those values produce the two flag columns. Therefore an implementation cannot compute the vector's actual result without inventing behavior.

## Coordination follow-up

Provide either:

1. An approved packet-producer contract with exact inputs, field shapes, the flag-generation rule, history read point, and concrete input/expected-output vectors; or
2. A later approved producer-boundary amendment treating both packets as opaque inputs from separately validated producers, including their lineages/receipts. The current D1 scope defers packet generation; `flags_before_history` remains `DEFERRED_TO_SOURCE_FLAGS_PRODUCER` and must not be reported as PASS or locally verified.

The separate local profile-binding test varies only the profile identity while holding provider lineage, state hash, and generation equal; the approved `profile_mismatch` vector only varies provider identity. D1 must implement only the complete persistent cold root and approved storage/staging foundation; P1.3-D stays `BLOCKED_ON_SOURCE_FLAGS_AND_FEATURE_PRODUCERS`.
