# Body-consumer range rule

Wash of a hash-bound static contract. The function reads a contiguous input buffer from argument 2. State argument 1 supplies unsigned start_index and six unsigned per-lane byte counts; lane capacity is six. If start_index < 6, the function reads argument 2 over [0, sum(count[i], i=start_index..5)), in ascending lane order, one byte per inner-loop iteration. If start_index >= 6, this function reads no body bytes.

Argument 3 is a raw 16-bit value stored to the ABI home area but not read again by this function. Equality between serializer length and argument 3 is only raw-value equality; equality to actual consumed bytes is NOT_EVALUATED. Runtime start_index/count values and actual consumed count were NOT_CAPTURED.

The function shows no explicit input-capacity or sum/pointer-capacity check. Caller storage must cover the computed range; malformed/short-input behavior is NOT_EVALUATED. The preceding serializer, same-parent, AU, source-generation, and selected-runtime joins remain UNBOUND. This conditional static contract does not establish that the function is the selected production consumer.
