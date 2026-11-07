# JOCForge selected-rate durable-append guard: 384 kbps first failure

## Run-bound observation

E-245 used the authorized Love Song N36/48 kHz/PCM24 source (SHA-256 `34ecda6afa708aaf436930b86cc8d120a73faff2847088be975b10a22039f8b5`) with selected `data_rate=384`. The run completed the full Meter prepass, committed g0/g1 priming and two suppressed source callbacks, then logged `source pipeline stopped: published source failed F12/F14/F15 durable receipt checks`. One 1,536-byte `.partial` file remained and no final EC-3 output was produced. The partial contents were not opened or hashed.

## Deterministic implementation blocker

In the current hash-bound `crates/jocforge-cli/src/programme_roots.rs` at line 592, the published-append validation unconditionally requires `append.bytes_written == 4096`. The selected 384 kbps profile is 1,536 bytes/AU (the six-rate profile map and E-245 frame-size receipt agree), so that predicate rejects the correctly sized first published AU. The same guard is therefore incompatible with every supported rate whose frame extent is not 4,096 bytes.

The narrow repair is to validate against the selected output/profile frame extent already admitted by `ProgrammeOwnerRoots::ready` (for example, `output_configuration().frame_bytes()`), while retaining the AU-index, CRC-zero, fsync, durable, metadata, F12 and F14 receipt checks. This is an output receipt invariant, not an audio transform change. The generic failure text prevented determining whether any additional F12/F14/F15 predicate would fail after this one is corrected; do not claim it is the only remaining run issue.

## Process/receipt caveat

The monitor receipt reports `EXITED_NONZERO`, while the child process exit code is 0; it also records the supervisor's stderr sharing-violation error. The application stderr independently contains the source-pipeline failure above. Treat the application attempt as failed with no output, and keep the monitor/exit-code discrepancy separate until fixed.

## Scope

No PCM, frame bytes, or payload were read by this Wash review. The run generated only the existing 1,536-byte partial file. No private key or license was accessed. No process was stopped, cancelled, or restarted. E-241 and the retained 1024 kbps baseline were not modified.
