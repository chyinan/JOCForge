# DEE native JOC delta, stride selector, and writer boundary

## Scope and disposition

This hash-bound supplement covers a local Atmos DDP JOC coefficient-payload path. It does not cover E-AC-3 core exponent syntax and does not explain the reported `exponent -1 is out-of-range` error.

Status: `CANDIDATE_RULE_READY_FOR_FOCUSED_IMPLEMENTATION_TRIAL; CURRENT_PROFILE_SELECTOR_AND_ROOT_CAUSE_UNBOUND`.

## Static native facts

The source image is `dee_audio_filter_ddp_atmos.dll`, SHA-256 `3d66bcec36031fd48e6565d15f05fea656642377ca4f8c98cdce1cce8b7e95d2`, analyzed with Ghidra 12.2.

- `FUN_1800942d0` calls `FUN_1800a3710` then `FUN_1800cb0d0` for the same block ordinal.
- `FUN_1800a3710` reads a block-local selector at `state + 0x140e8 + block * 10 + 0xd8`. Selector `0` skips this difference branch. For nonzero selectors, it indexes the first four observed table entries `[0, 1, 2, 4]`: selector `1` selects stride `1`, selector `2` stride `2`, selector `3` stride `4`. The use site has no range check; values above `3` are unsupported by this evidence.
- The selected short transform in `FUN_1800a4d70` emits `current - previous + 2`, advancing by the selected stride, and fills `(3 * total_slots - valid_count)` tail entries with `2`.
- `FUN_1800cb0d0` reads two width-7 values from the block pointer-table expression based on `param_11 + 0x3b8 + block * 8`, with the sequence beginning at byte offset `2`, then reaches `FUN_18008aea0`.
- `FUN_18008aea0` masks the requested width, inserts high bits first into a 16-bit accumulator, flushes when the cursor exceeds 15, and retains residual bits. `FUN_180093220` writes one masked fixed-width value; `FUN_180093360` appends packed words and residual bits.
- The public wrapper accepts process block size `0x600` samples (1536) or `0` for flush.

## Best-supported implementation trial

Preserve the block-local selector and apply the observed table mapping independently per block. Do not infer the selector directly from the `1024 kbps / 5.1` label. For the JOC payload candidate, isolate the short-value delta/tail-fill rule and fixed-width writer against Rust's F32 → modular delta → F05Q Huffman path. This is ready for a focused trial without waiting for a perfect pointer alias or runtime profile join.

Confidence is high for the local selector/table and transform mechanics, medium for associating the width-7 field with the preceding producer, and low (about `0.3`) that this caused the reported Rust parse/playback failure. The exact 1024/5.1 selector, selected route to this frame assembler, and producer/packer pointer alias remain unobserved. The most likely alias is the per-block record at `state+0x140e8+0x2c8+block*0x28` holding the transformed shorts while the adjacent table at `state+0x140e8+0x3b8+block*8` caches a second view; confidence about `0.45`, not a proven fact.

## Required boundary

This is a JOC payload candidate only. Keep it separate from the E-AC-3 core exponent path. The source package used one hash-bound DLL, zero authorized ADM experiments, and zero DEE runtime observations. It contains an input-only synthetic vector; no expected bytes or output values were included or executed.

## Exclusions

No source code, DEE binary, disassembly, decompilation, media, PCM, private key, or license material is included. Hash provenance and summarized facts are in `EVIDENCE.json` and `SOURCE-MANIFESTS.json`.
