# F03C candidate selector direct slices

This isolated F03C module exposes three stateless primitives over caller-supplied values: pairwise affinity, primary/secondary score choice, and the cold candidate selector. The selector consumes same-callback feature/position/response/width/attribute arrays and previous target IDs, then returns selected global IDs, original ordinals, attributes, initial affinity/complement/support, and per-round gate/tie/mask/coverage/survivor diagnostics.

The implementation follows the supplied ordinal order and F32 rounding points. It does not produce feature FFTs, geometry responses, U04 targets, target installation, callback metadata history, or a cross-call selector state. The spec's `pow32`/`cos32` mathematical route uses the platform F32 math operations; matching these vectors is local evidence only, not real-audio spatial acceptance.

The local test selects exactly eight F03C entries from the approved unit-vector asset: four full-selector cases, three score-choice cases, and one pairwise-affinity case. No input vector or expected output is used as production state.
