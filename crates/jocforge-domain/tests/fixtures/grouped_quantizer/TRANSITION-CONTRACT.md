# Grouped quantizer reservation transition

This amendment is limited to valid BAP values 1, 2 and 4 and the observed local helper state.

## Item reservation

Each quantized item first reserves its field width at the current logical tail and increments the effective-bit count. The item remains in its group state until that group reaches its fixed arity:

- BAP1: three items, width 5;
- BAP2: three items, width 7;
- BAP4: two items, width 7.

An incomplete group therefore retains its count and its reserved zero/value bits. The `incomplete_groups_no_flush` vector records three counts of one, 19 effective bits, two emitted bytes and three pending bits with no packed field.

## Group completion and compaction

When a group reaches arity, its reserved item slots are compacted in place to one packed field at the first item offset. The later reserved fields shift left by the removed reservation widths. The effective-bit count is reduced by the removed item widths. The pack event records the resulting field's `start_bits`, value and width; the group count then returns to zero.

The observed complete-group vectors show packed starts at 0, 5 and 12 for the BAP1, BAP2 and BAP4 fields respectively. The packed values are produced by the radix rules in `SCOPE.md`.

## Tail flush

Flush fills only the currently reserved incomplete groups, then compacts each filled group in place. It does not reserve a second copy of the group. BAP1 fills with code 1, BAP2 with code 2, and a single BAP4 item with code 5. Flush returns zero and clears the three group counters.

## Word memory and pending bits

The helper reports emitted word-memory bytes separately from its pending accumulator. The supplied vectors are authoritative for the observed alignment: effective bits are split into emitted bytes and pending bits, with the emitted word-memory byte order and pending accumulator compared exactly by vector hash. These bytes are not a complete EC3 stream and do not establish a caller capacity contract.

The helper has no capacity argument, status result, or rollback. Valid-domain overflow/capacity ownership and higher-level retry restoration remain outside this contract. BAP values outside 1, 2 and 4 remain unobserved and rejected by scope.

