# Opener table producer and setup-pointer gap

This is an additive Wash-only static lineage note. It independently hash-checks the source package manifest and both declared artifacts, then reconciles them with the existing washed CHNA row-slot and setup-record boundaries.

## Typed row/payload distinction

For the analyzed opener path, CHNA lookup supplies a one-based TrackIndex; subtracting one selects the zero-based opener table row slot. Object payload is an AudioObjectID-derived source-channel key, not the TrackIndex. Bed payload and label come from the bed-channel descriptor. The six-byte table producer is statically identified. This agrees with the existing washed channel-table package.

## Setup pointer remains open

The separate setup callback is registered indirectly. The scoped static search found no transfer from the opener-owned table pointer to setup `param4[2]`. In the earlier same-sample run, the attempted module-plus-relative-address breakpoints did not resolve and neither hit. No pointer values or owner/generation tokens were captured; pointer equality is therefore NOT_EVALUATED. Neither the failed instrumentation nor the static non-find proves pointer inequality or event absence.

A future authorized runtime observation should use the exact converter image identity and qualified module coordinates from the research package, then compare pointer tokens only within the same input owner and parent generation. This supplement does not authorize another DEE run and does not change an audio algorithm.
