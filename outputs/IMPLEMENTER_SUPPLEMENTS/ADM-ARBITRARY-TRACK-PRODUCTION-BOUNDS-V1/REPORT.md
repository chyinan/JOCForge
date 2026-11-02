# Arbitrary-track ADM production bounds

**Status:** the first current-source blocker for the authorized 36-row input is the Meter sample-extent/callback root; subsequent fixed shapes are inventoried. No complete 36-row JOCForge encode was run.

## First concrete blocker

The existing 36-row metadata receipt binds 7,328,771 samples per channel. In the current production path, `StandaloneMeterRoot` still pins 6,522,906 samples, 4,247 normal callbacks, three zero drains, and fixed energy-window/remainder state. `meter_prepass.rs` constructs this root before the PCM cursor. Thus the 36-row input cannot pass that exact current sample-extent check. The report derives, but did not execute, a 36-row expectation of 4,772 normal callbacks and 1,021 final-frame padding samples; with three retained Meter drains the Meter callback total would be 4,775.

The Meter PCM cursor also has a `[36, 38]` allowlist, while its `N × 3` block-align validation and per-row allocation are already dynamic. Remove the allowlist only when the selected graph/profile rules for the wider `N` range are explicit; do not replace the existing tail/remainder rules with an unverified rounding formula.

## Remaining source constraints, in execution order

- F01E production still uses a legacy 38 physical / 10 bed / 28 object / 37 regular snapshot even though a graph-sized owner/proposal API exists. The selected profile has 10 beds; regular rows must be tied to the graph's direct LFE identity.
- Spatial sources retain a 48-wide global axis with 11 prefix rows, 38 physical hosts and 37 regular hosts. The existing 36-row receipt maps 35 non-LFE rows to candidate IDs 11–45 and leaves IDs 46–47 absent. Do not zero-fill or alias those ids without evidence. The `11 + (N − 1)` mapping currently fits that axis only through N=38.
- Meter activity, raw/previous PCM rows, controls, regular histories, EOF, and drain carriers still contain 38/37 shapes. Preserve the existing three Meter FreshZero drains and four MAIN cached drains while passing one admitted dynamic row identity through them.
- F15 derives extents from samples, but the selected provider still has 4,249 dispatch and 4,247 published-count checks. The same-run clock join for a real 36-row job is unverified.
- The Rust source bridge does not yet populate a DDP per-call frame-view channel count. The native frame-view count and active capacity are not typed to ADM rows. Do not treat the adjacent native `root+0xC140` capacity as a physical-track limit until that join is bound.

No inspected 38/114/sample-count/48-source constant is proven to be an external ABI limit on the current JOCForge CLI. The current internal 48-wide axis is a real model constraint; support beyond 38 requires a deliberate redesign across all consumers. This package gives a concrete 36-row implementation order without claiming arbitrary-N E2E success.

## Next 36-row validation

The future command and expected product are recorded in `NEXT-VALIDATION-VECTOR.json`. It was **NOT_RUN**. Static expectation: 4,772 published AUs × 4,096 bytes = 19,546,112 bytes; this is not an observed output, and a complete encode, publication receipt, decode, or playback is not established.

## Evidence scope

Wash reran the source package verifier in normal Python and `python -O`: four package files, 31 source files, and seven existing text-evidence hashes passed. The package is bound to current HEAD `b93b9a34016c0cd669de1e82a3f2ee5466518b02`. This is a static source audit plus reused metadata receipts only; it ran no CLI, DEE, tests, or audio decode and read no PCM/data payload/key/license.
