# DEE LFE stride and transform-input boundary

## Implementation-facing finding

The LFE recurrence branch depends on the selected core descriptor's typed U16 sample stride at `descriptor[i] +0x0C`. `FUN_140317600` copies descriptor strides into `state+0x1320`; the layout map selects the LFE descriptor as `layout_id*6 + FBW_count` when LFE is present. The selected value reaches the LFE wrappers and chooses `FUN_14036d9e0` behavior: zero uses the inline recurrence; nonzero enters the distinct `FUN_14036d7c0` stride-table branch.

The native recurrence does not consume the ADM row pointer directly. The selected descriptor pointer is gathered into a 256-F32 scratch area, passed through the shared transform `FUN_14036e350`, then the resulting 256-F32 temporary is passed to the LFE wrapper and transformed in place. Rust F08 receives a folded 256-F32 input with the same shape, but no same-owner/generation join is established.

## Decision status

The field and static transform stage are bound. The selected stride/layout for an authorized 5.1 sample is still `UNBOUND`; zero/nonzero branch selection is not observed. Numeric parity and playback were not run. This is an actionable candidate path, not a proven explanation of the audible difference. Do not feed a raw BWF row directly into the core LFE recurrence or assume the zero-stride branch without one sample-level field observation.

The Rust callsite and HEAD recorded in the source package are historical package context (`613112b7fe158052500244d1af202817bcf9f216`); recheck current implementation HEAD before using the comparison as current-code evidence.

## Minimal next observation

Use one run-scoped metadata-only capture: descriptor pointer tokens and stride values, outer/normalized layout fields, selected layout row, FBW count, LFE lane/slot, and the shared-transform temporary pointer token at the LFE wrapper. Repeat on another authorized ADM if the first sample exposes an unambiguous branch. Do not persist PCM or read private keys.

## Source package

Wash independently checked `DEE-LFE-SELECTED-STRIDE-INPUT-EDGE-WASH-V1`: final `MANIFEST.json` SHA-256 `fd6ec172d610ff94867cec38d937396cdd9444f6d05c0921d0392d34e454f2bd`; `package_seal.json` SHA-256 `de81a8d51dc7bba9ae755add13a11a5c1858384be0f298bb1e32153d8abc8a98`. The source package checker passed in normal and optimized Python modes. Its scope says no DEE run, PCM/media/payload/key access, numeric parity, or playback.
