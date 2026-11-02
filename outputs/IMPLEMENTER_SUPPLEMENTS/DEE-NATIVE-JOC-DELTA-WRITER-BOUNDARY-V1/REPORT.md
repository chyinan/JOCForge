# DEE native JOC delta and writer boundary

## Scope and disposition

This is a hash-bound implementation supplement for the native Atmos DDP JOC coefficient-payload path only. It does not describe E-AC-3 core exponent syntax or establish why the reported `exponent -1 is out-of-range` error occurred. No DEE run, ADM experiment, or audio comparison was performed in the source package.

Disposition: `CANDIDATE_RULE_READY_FOR_FOCUSED_IMPLEMENTATION_TRIAL; ROOT_CAUSE_NOT_ESTABLISHED`.

The supplement supports trying the native local transform and writer behavior in the current 1024 kbps / 5.1 JOC path. Do not wait for a perfect pointer-owner proof before running the focused trial. Keep the observed facts and the unobserved producer/consumer join separate.

## Hash-bound native facts

The source is `dee_audio_filter_ddp_atmos.dll`, SHA-256 `3d66bcec36031fd48e6565d15f05fea656642377ca4f8c98cdce1cce8b7e95d2`, analyzed statically with Ghidra 12.2.

- In the local block path, `FUN_1800942d0` calls `FUN_1800a3710` and then `FUN_1800cb0d0` for the same block ordinal.
- `FUN_1800a3710` calls `FUN_1800a4d70`. The static rule is: for each selected short, emit `current - previous + 2`, advancing input by a table-selected stride; fill `(3 * total_slots - valid_count)` tail entries with `2`.
- `FUN_1800cb0d0` reads two width-7 values from a pointer-table expression based on `param_11 + 0x3b8 + block * 8`, with the value sequence beginning at byte offset `2`, then calls `FUN_18008aea0`.
- `FUN_18008aea0` masks to the requested width, places high bits first into a 16-bit accumulator, flushes a word when the bit cursor exceeds 15, and retains residual bits. `FUN_180093220` writes one masked fixed-width value; `FUN_180093360` appends full words from an existing packed stream and then appends its residual bits.

## Comparison and confidence

The hash-bound Rust source path in the incoming package is F32 coefficient rows → coarse/fine quantization → modular deltas → selected F05Q variable-length Huffman codewords → ID14 body append. The native local path above uses short values, a linear delta plus `2`, tail fill with `2`, and fixed-width fields. This is a concrete representation-mismatch candidate for JOC payload data.

Confidence is high for the local native arithmetic and bit-writer behavior; medium for associating the width-7 field with the immediately preceding producer; low for claiming this caused the user-reported damaged output. The producer writes through `state + 0x140e8 + 0x2c8 + block * 0x28`, while the packer reads through `param_11 + 0x3b8 + block * 8`; pointer alias, selected route to frame assembler, and candidate-file/run identity remain unbound. The source package used zero authorized ADM samples and zero DEE runtime observations.

## Action for the implementation trial

Use this as a focused JOC-payload trial on the current 1024 kbps / 5.1 path: implement the stated local delta/tail-fill and MSB-first fixed-width writer behavior where it matches the active payload block, then run it against at least two user-authorized ADM inputs. Check whether the output parses and whether the JOC payload advances without changing the separate E-AC-3 core exponent path. If the target field does not map to the Rust F05Q payload, record that result and continue from the first actual DEE/Rust JOC body boundary; do not treat this package as a fix for the core `exponent -1` error.

The attached vector is input-only and unexecuted. It intentionally contains no expected output bytes or asserted buffer alias.

## Exclusions

No DEE/implementation source code, binary, disassembly, decompilation, protected media, PCM, codec output, private key, or license material is included. The evidence is limited to the summarized static facts and source hashes in `EVIDENCE.json` and `SOURCE-MANIFESTS.json`.
