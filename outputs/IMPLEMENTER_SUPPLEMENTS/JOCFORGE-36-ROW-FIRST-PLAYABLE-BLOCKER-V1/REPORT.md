# First-playable 36-row CLI blocker

## Wash conclusion

The current CLI does not stop at the public application API stub. It uses the CLI source-execution path. On the audited HEAD, the first concrete 36-row rejection is the fixed-width Meter prepass, followed by a fixed-width F01E callback root that must be converted to dynamic state before a 36-row spatial callback can run.

## Ordered blockers

1. `MeterPcmSourceCursor::cold` calls a selected-source validator requiring 38 channels and `block_align=114`. A 36-row PCM24 input at 48 kHz has `block_align=108`, so the CLI returns `SourceProfileMismatch` in Meter prepass before the main spatial callback and before any EC-3 frame.
2. Input admission and Meter source-owner admission are already dynamic by actual WAVE/graph row count; F15 parent context uses selected profile and sample count and has no 38-channel gate. The failure is in the Meter physical-row cursor/view shape, not basic RIFF/ADM admission.
3. After Meter is dynamic, the live CLI callback still creates fixed `MainSpatialColdRoots` and calls the legacy F01E normal-control snapshot/proposal path (38/10/28/37). A dynamic 36/10/26 normal proposal exists but is not installed/committed into the live spatial root.

## Smallest useful vertical slice

- Make Meter normal/fresh-zero source views accept the actual physical rows from the same `ParsedRiffPcm24` and `PreparedJobInput` graph; retain the existing Meter algorithm's output lanes and clock.
- Route the live CLI callback through the dynamic F01E 36/10/26 owner and normal/fresh-zero proposal, installing its after-state into the same `ProgrammeOwnerRoots` source-advance transaction.
- Reuse the existing spatial/F07-F15 candidate path, durable AU append, and F15 finalize/expose path after those two gates. Do not change fixed output/codec profile dimensions or map input-row count to F13 coefficient-slot extent.

The `application::encode()` API stub is a separate entry path; changing that stub alone does not fix the CLI. Current completion logs still say render/playback `NOT_RUN`. A real 36-row run, published AU, final output exposure, independent E-AC-3/JOC parsing, and Audition/decoder playback remain required for a playable-output claim.
