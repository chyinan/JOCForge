# Spatial control staging specification gap

## Status

`P1.3-D1-SPATIAL-CONTROL-STAGING-ONLY` is blocked before Rust implementation. The v5 supplement is identity-verified, but it does not fully specify the `SourceFlagsPacket` producer required by one of its input-only vectors.

## Evidence

The approved `flags_before_history` case in `crates/jocforge-domain/tests/fixtures/spatial_control_state_v5/INPUT-ONLY-VECTORS.json` provides:

- `same_before_history: true`;
- `different_current_input_or_writers: true`;
- expected relation `flags_may_differ: true`;
- expected relation `next_history_read: false`.

It does not provide concrete callback input/control/route/writer-set records, the rule producing `flags: U8[48][2]`, or concrete expected flag bytes. The v5 contract says the packet is calculated from before history and same-callback input/control/route/prefix writers, but not how those values produce the two flag columns. Therefore an implementation cannot compute the vector's actual result without inventing behavior.

## Minimum unblock

Provide either:

1. An approved packet-producer contract with exact inputs, field shapes, the flag-generation rule, history read point, and concrete input/expected-output vectors; or
2. An approved D1 scope amendment treating `SourceFlagsPacket` as opaque input from a separately validated producer, including its lineage/receipt, and explicitly deferring packet generation plus the `flags_before_history` producer test.

Until then, keep `spatial_control` unimplemented and do not create a partial root. The separate local profile-binding test must vary profile identity while holding provider lineage and state hash equal; the approved `profile_mismatch` vector only varies provider identity.
