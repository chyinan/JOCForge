# F09 D45 cold-state and route boundary

## Static rule for the traced owner

The hash-bound `dee_audio_filter_ddp_single_pass.dll` constructor clears five 8-byte lane records at owner `+0x135C0`. The traced history-copy routine copies each lane's first U16 to a six-block-by-five-lane history at owner `+0x15208` only on its `param_1 == 0` branch and under the lane-count gate. The producer search found no later direct store to the source words in the seven scanned images. This supports a cold-zero initialization for this typed owner path; it does not exclude indirect or unscanned writes and does not join the owner to the selected ADM task.

## Route boundary

Ghidra connects the registered `FilterDDPEncodePlugin` through its `DDPEncode` workflow, direct and queued `Comp` branches, and the `0x600`-sample-or-flush guard to the D45 history path. Runtime attempts 10/11 did not hit the target owner/history breakpoints. Attempt 12 loaded the hash-matched plugin but its WAVE input stage failed before PCM-frame delivery because the required `MediaInfo.exe` was unavailable at the effective plugin root. No runtime owner/history/force observation is claimed.

The attempt-08 full JOC output and post-rename F15 failure are a separate JOCForge run; they do not prove this DEE D45 path. The exact same-root MediaInfo/DEE/plugin combination is still unavailable in the four checked roots. No substitute MediaInfo root or global environment change was made.

## Disposition

Reference only; `implementation_ready=false`. The static cold-zero rule is bounded to the typed constructor path and is not joined to the selected R138 owner. Later nonzero reachability, selected route, runtime force/history values, and the F09.4 native result remain `UNBOUND` or `NOT_RUN`. No DEE algorithm rule is inferred from another encoder.

No PCM or EC3 payload was read. The source package read no private license/HMAC/key contents. Its seven-image search is a bounded scan, not a global absence proof.
