# DEE versus current production Rust F08 LFE coefficient candidate

## Wash conclusion

This is the first hash-bound numeric parameter difference candidate in the selected Rust F08 LFE production path. Wash independently verified that the current implementation at HEAD `613112b7fe158052500244d1af202817bcf9f216` still calls `f08_lfe_transform_block` after the F08 window fold, and that its current parameter JSON hash is `963a176760f7c7d43dba138553717b7d5273a9457a574f658ee73eeb62e7da5c`. The current Rust recurrence/cosine/sine arrays match the R4 crosswalk's Rust arrays. The values below are F32 bit patterns.

## Concrete coefficient deltas

The DEE eight-word recurrence/precursor vector versus current production Rust `lfe_recurrence_f32_bits` matches indices 0–4 and differs at 5–7:

| Index | DEE | Current Rust |
|---:|---|---|
| 5 | `3ffd3aac` | `3ff853f8` |
| 6 | `3ffec46d` | `3ff4fa0b` |
| 7 | `3fffb10f` | `3ff10908` |

The DEE main finalizer versus current Rust cosine array matches 0–4 and differs at 5–7. The DEE companion finalizer versus current Rust sine array matches 0–3 and differs at 4–7:

| Index | DEE companion | Current Rust sine |
|---:|---|---|
| 4 | `be47c5c2` | `3e47c5c2` |
| 5 | `be164083` | `3e78cfcc` |
| 6 | `bdc8bd36` | `3e94a031` |
| 7 | `bd48fb30` | `3eac7cd4` |

DEE's recurrence output is scaled by `1/16` (`3d800000`) before terminal seven-bin rotation; the terminal seven-bin functions themselves do not apply that scale. The current Rust F08 recurrence scale is also `3d800000`.

## Direct matches and pathway distinction

- The zero-stride pre-rotation formula and 128-word long cosine/sine tables match Rust `long_pre_rotation`.
- The DEE gather permutation and 8-point complex butterfly operation match Rust F08; the terminal seven-bin primary/companion output formulas also match.
- A separate Rust helper, `lfe_transform_to_raw`, contains the native-matching arrays, but the production owner calls `f08_lfe_transform_block`; the helper is not the selected production callsite.
- DEE's selected stride branch remains unknown. Its nonzero-stride LFE path uses two 64-word tables; the Rust lane-3 spatial filter is a distinct 20-word/four-section stateful filter with no direct coefficient-table or SOS identity shown here.

## Limits and actionable candidate

This supports testing the native bit patterns in the three current F08 parameter arrays, while keeping the selected stride branch and input buffer identity explicit. It does not establish that ADM physical row 3 reaches the same native core LFE lane, nor that the buffers/generations match. Numeric parity and audio playback were not run. Treat the three array differences as a high-priority candidate for a controlled multi-ADM audio comparison, not an already proven cause of the damaged output.
