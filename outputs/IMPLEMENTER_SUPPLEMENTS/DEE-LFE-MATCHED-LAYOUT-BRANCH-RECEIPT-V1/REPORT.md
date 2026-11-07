# LFE matched-run layout and stride branch boundary

## Static path

The hash-bound Ghidra 12.2 trace binds one static selector edge. `FUN_1403471f0` copies the per-stream context's two 32-bit fields at `lVar9+0x40/+0x44` through helper `param_1+0x34/+0x38` into core state `+0x2c/+0x30`. The fields are used as layout ID and LFE-present inputs, then normalized to `state+0x1912c/+0x1912e`.

The static layout path is `FBW_count = DAT_1405f4ba0[layout_id]`; with LFE present, its lane is `FBW_count`; the descriptor slot is `DAT_1405f4bc0[layout_id*6 + lane]`. `FUN_140317600` copies the selected descriptor U16 stride at `descriptor+0x0c` to `state+0x1320[slot]`. The stride reaches the LFE main/companion wrappers: zero selects an inline recurrence; nonzero reaches `FUN_14036d7c0`'s stride-table branch.

## Matched-run boundary

Existing receipts bind the Human Cat ADM input identity `7826dcbef37ef69642a6164c97a37bcfb3f917cea74aa0f0a30d76d0a825f1d1`, matched XML hash `c44aaaf5d155d2bc65d67d127ca48c43a5e58402643ffa74169c7f73a9365bc7`, DI=false, threshold 100, and 48 kHz six-channel 5.1(side) geometry. A separate success receipt reports 38 input rows and output SHA-256 `80c3b9b9a3ed1eaa74a53dbd5bbdc714b0f00c1501e44e4c43eb1aa4ca9a2662`; it does not establish which selector branch ran, and the output was not opened or locally rehashed in that receipt.

The matched run does not record dispatch mode, layout ID, LFE-present value, FBW count, descriptor slot, or U16 stride. Therefore, the static mapping is bound but the matched-run zero/nonzero branch is `NOT_CAPTURED`. The exact first break is the hash-bound input/XML/output geometry to the runtime core context at `+0x2c/+0x30` and selected descriptor stride. No LFE residual root cause is established; the zero-stride recurrence remains only a conditional match to the existing Rust rule, and the nonzero path has no direct match in the crosswalk.

## Boundaries

Source manifest SHA-256: `56d4dfa6a0acdd9a0115667de5738393e588b2c137ba0c39e366635cefa8c34d`; upstream manifest checksum file matches. Its normal and `python -O` verifier both passed. No separate package seal was supplied. No DEE run, PCM/media/payload/key/license read, or implementation contact occurred. The old `sample6-keyless-full-20261101-08.ec3` artifact was not used. This supplement carries no audio or binary content.
