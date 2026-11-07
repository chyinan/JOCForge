# DEE LFE zero-inline coefficient table digests

The corrected selector receipt states that the two inspected dispatcher paths assign selector zero for the LFE lane, selecting the inline branch in `FUN_14036d9e0`. A separate hash-bound Ghidra 12.2 memory export provides digests for that branch's two coefficient regions:

| Table | Elements | Bytes | Raw-byte SHA-256 |
|---|---:|---:|---|
| zero-branch table A | 128 binary32 | 512 | `bbd2bd460b0a750ccb45f959561a9a7e2ed2f8c6fd9391fb21abc9216b08fa41` |
| zero-branch table B | 128 binary32 | 512 | `e4cbb855ed455887b387cdb188bf8bab41976c73648c1b022dd18156214bde50` |

These are native-table identity digests only. The sealed F08 candidate's report says its long tables match, but its package does not contain either corresponding 128-word candidate array or per-table digest. Direct A/B comparison is therefore `NOT_RUN`; no match or mismatch is claimed. The table values, candidate comparison, sample-specific layout, numeric parity, and the reported LFE correlation cause remain open.

No DEE run or PCM/payload/key/license read occurred. No implementation source was inspected.

## Provenance

Source addendum `dee_lfe_zero_inline_coefficient_f08_comparison_wash_v1` manifest SHA-256 `da136aac97923622f8035428030c42663c79b145ed2216ec263e19c0e961acaa`, package-seal SHA-256 `20e3c3d1d980002dfa801fd5f5e1ffd438a4db15544ac174edcf44bcf144c7b7`; its normal and `python -O` checkers passed.
