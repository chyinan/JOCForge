# F06OF local Meter drain rule versus native zero-length submissions

Disposition: LOCAL_FRESH_PACKET_RULE_DEFINED; NATIVE_DRAIN_METADATA_JOIN_NOT_CAPTURED. This supplement separates the selected Meter contract from bounded native observations.

## Provenance and verification

Source package METER-DRAIN-METADATA-INSTALLATION-V1 manifest SHA-256: 9949acae6775b4b73bcd511e35fd89bb67306b33aa8103a559ff1bc70e8e6662.
DRAIN-METADATA-STATUS.json SHA-256: 174566d296b643539543d1179a32be3fcc12e63aef88c9d762ed7f0761419365.
INPUT-ONLY-VECTORS.json SHA-256: 1b52af65fd6830fde9f558a3cab3941934154597164977fb225d0dc9fdd00bdb.
Source manifest, all members, and checker passed normal and optimized Python.

## Approved local F06OF drain behavior

For each Meter drain g>=Q, do not read new source. Keep cursor N, create a fresh positive-zero raw 38x1536 frame, retain frontend_samples=1536, advance the full frontend stage, and produce a new same-generation SCP16 (16x1536 finite F32) plus control16 (16x13 U32). Do not use the previous SCP/control as a substitute. The main F01E four cached drains are a distinct owner and clock.

The contract does not yet prove whether the raw ADM event snapshot itself is reinstalled or retained during each Meter drain. It requires the frontend producer to execute and emit a new packet; exact source metadata installation remains open.

## What the native trace observed

The referenced native run reported 4,247 normal source records. It separately observed one source-empty candidate at generation 4248 with source_count=0, one frontend call, 6x1536 renderer output, and one 1536-sample Meter input. It then observed three downstream zero-length Meter submissions (indices 424:éÝyø§yÓ4251, channels=0, samples=0), each associated with the integrated owner and ordered finalizer/getter calls.

Those three submissions have input_call_generation=null; no control/SCP rows, F01E snapshot token, or same-run join from the source-empty candidate to the finalizer submissions was captured. Do not label them as the three F06OF fresh-zero frontend drains.

## Exact missing edge and next observation

Still unbound: Meter input/event-control owner -> DRAIN stage marker/metadata-history producer -> newly produced control16/SCP16 -> OAR/F06OC packet consumer, with same attempt and generation.

Next bounded observation: at the last normal callback and three Meter DRAIN callbacks, capture only phase/generation, N/cursor, event or metadata owner token, control-table owner/shape/hash, and packet token at both frontend output and OAR input. Then correlate DPLM finalizer submissions through caller stack/owner if they are asserted to correspond to those drains. Do not read PCM.

No production root install, parent swap, READY, AU, output, playback, or product acceptance is claimed by this supplement.
