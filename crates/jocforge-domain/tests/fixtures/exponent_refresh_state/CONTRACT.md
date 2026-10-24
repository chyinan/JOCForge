# Exponent refresh state transition

## Scope

This contract covers one in-place exponent-row refresh helper. It does not produce the initial POWER-to-exponent row, select the caller's mode or marker, load the upstream bias/configuration, or define the caller's reset/EOF/failure lifecycle. Those parts of F09 remain open. This package is ready only for the bounded helper described below; it does not make the full F09 producer implementation-ready.

## State and inputs

- `current` and `work` are caller-owned, equal-length rows of signed I16 values. The rows are disjoint.
- `marker` is caller-owned signed I16 state. The native branch is selected by `marker != 0`.
- `mode` is a 16-bit operand. Zero selects ordinary refresh; nonzero selects weighted refresh. The input vectors use only 0 and 1.
- `special` is signed I16. The special threshold is selected only when `mode != 0 && special > 0`. The input vectors use only 0 and 1.
- `begin` and `end` are signed I16 indices, with `end` exclusive.
- One invocation updates the supplied `[begin, end)` window. Repeated calls retain all mutations; this helper has no reset, transaction, or rollback operation.

The bounded slice preconditions are: `mode` and `special` are each 0 or 1; `0 <= begin < end <= row length`; `end < 32768`; all initial values in both rows are in `0..=31`; and the selected branch's every difference-table index is in `0..=31`. The caller must establish these conditions. The native helper does not validate them, and behavior outside them is unknown. A Rust `bool` or enum for the two 0/1 selectors is a Wash interface choice, not a recovered native typedef.

## Parameters

Use the exact bytes in `PARAMETERS.json`:

- Difference table: 32 signed I16 values in little-endian order: `0, 0, 1, 2, 4, ... 58`.
- Position weights: four signed I32 values `[3, 6, 10, 10]`.
- Normal threshold: F32LE bytes `23 a1 bd 3e`, bits `0x3ebda123`, value `0.3703700006008148`.
- Special threshold: F32LE bytes `3d 0a d7 3e`, bits `0x3ed70a3d`, value `0.41999998688697815`.

## Transition order

The `marker != 0` branch has precedence over `mode`:

1. Set `work[begin] = current[begin]`.
2. For each `i` from `begin + 1` through `end - 1`, sign-extend `work[i - 1]` to I32, add 2 in I32, and set `work[i] = min(current[i], limit)`. `current` and `marker` remain unchanged.
3. Do not read the threshold or difference table in this branch.

When `marker == 0 && mode != 0`, use weighted refresh:

1. For each `i` in `[begin, end)`, starting at `begin`, compute a table index from the current and work rows. For `i > begin`, first clamp `current[i]` against `I32(current[i - 1]) + 2`; if the signed I32 comparison `current[i] >= limit` succeeds, write the low I16 of `limit` back to `current[i]`.
2. Compute `index[i] = trunc_toward_zero(abs(I32(current[i]) - I32(work[i])) * weight[min(i, 3)] / 10)`. The weight uses the absolute row index `i`, not `i - begin`.
3. Add `difference_table[index[i]]` to a signed I16 accumulator. Each addition wraps modulo 65536, then is interpreted as signed I16.
4. Select the special threshold only when `special > 0`; otherwise select normal. Compare `F32(sum * 2^-15) > F32(F32(I16(end - begin) * 2^-15) * threshold)` using the stated F32 operation order.
5. If the strict comparison is true, set `marker = 1` and copy `current[begin..end]` to `work[begin..end]`. Otherwise leave `marker` and `work` unchanged. Any clamps already applied to `current` remain in either case.

When `marker == 0 && mode == 0`, use ordinary refresh:

1. Initialize the signed I16 accumulator with `difference_table[abs(I32(current[begin]) - I32(work[begin]))]`, then set `work[begin] = current[begin]`.
2. For each later `i`, sign-extend `work[i - 1]` to I32 and add 2 in I32. Select `value = min(current[i], limit)`; the native comparison is strict `current[i] > limit`. Accumulate `difference_table[abs(I32(value) - I32(old_work[i]))]` with signed I16 wrapping, then write `work[i] = value`.
3. Compare with the normal threshold using the same F32 operation order and strict `>` comparison. If true, set `marker = 1`. The work row has already been updated regardless of this comparison. `current` remains unchanged.

No explicit status or error behavior is established. The state changes are incremental and are not rolled back. Do not invent a native rejection result for a violated precondition; any local Rust admission error must be labeled as a separate API design choice.

## Evidence and limits

The static transition and operand rules are `STATIC_CONFIRMED`. The native runtime package covers 10 input recipes and 12 state transitions; its prototype postcompare is 12/12. Wash independently replayed the formula against all 12 expected transitions, including a pair that distinguishes the normal and special thresholds. This is not an independent product implementation or whole-chain validation.

Not covered: arbitrary row domains, invalid pointers/ranges, aliased storage, nonzero-begin multi-bin weighted refresh, exhaustive table indices, threshold-adjacent rounding, caller mode/marker production, upstream POWER/bias/configuration, or parent reset/EOF/fatal behavior. Keep those as open F09 work.
