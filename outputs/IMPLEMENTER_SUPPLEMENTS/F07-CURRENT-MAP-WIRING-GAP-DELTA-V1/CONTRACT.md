# F07 current implementation-map wiring gap delta

Status: `F07_1_PRIOR_WASH_CLOSURE_RETAINED`; `WHOLE_F07_F07A_F08_WIRING=OPEN`.

This delta supersedes the implementation-map snapshot in the earlier F07 gap audit, not its F07.1 rule closure. It records the current hash-bound map state and leaves only the remaining integration mappings.

## Current mapped state

- F07.1 per-lane block history remains a locally verified direct slice; whole F07 is deferred. The current map also records direct local slices for structure-refresh flags, selected dither planning, and one actual-D05 transient channel block. They remain component slices, not a complete frame owner.
- F07A has a hash-pinned D05 provider slice; it does not perform all-lane transient filtering or parent handoff.
- F08 has a locally verified F08.2 two-bank fold/history slice; pre/post rotation, FFT, LFE, parent installation, and whole F08 remain outside the current direct map.

## Three remaining production wiring gaps

1. Bind the six `installed_audio` rows to a production source/frame owner and generation, and bind the corresponding F07 commit/receipt to that source.
2. Connect F07A transient consumption and state advancement across five FBW lanes plus LFE, including the F07.1 input, queues, F07.4/.5 and parent owner.
3. Map the full delayed-audio path through F08 and its enclosing state owner beyond the F08.2 fold/history leaf.

The analysis fixture seam remains test-only. These gaps describe missing production integration, not missing F07.1/F08 equations. No tests or native execution were performed in this delta.
