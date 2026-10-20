# Analysis owner projection canonical hashes

The four owner digests use no binary wrapper and no domain prefix. Each
preimage is one field-complete JSON object encoded as UTF-8 with keys sorted
lexicographically, comma/colon separators, ensure_ascii=false, no whitespace,
no BOM and no trailing newline. Array order is preserved.

All binary32 state is represented by its exact U32 word as a JSON decimal
integer. U16/U32/U64 values are JSON decimal integers in their declared range.
I1 queues are JSON integers 0 or 1, not booleans. Non-finite JSON numbers and
compact repeat descriptors are forbidden.

## input_history

Fields are generation U64, then history_u32_words with exactly 7680 U32 words.
Canonical JSON key sorting determines byte order; the semantic field list does
not authorize another serialization.

## transient

Fields are generation U64, two_history_u32_words[20],
three_history_u32_words[30], peak_statistics_u32_words[90],
previous_mean_u32_words[5], short_queue_i1[20], other_queue_i1[20].

## primary_companion_lfe

Fields are generation U64, primary_fbw_history_u32_words[640],
companion_fbw_history_u32_words[640], primary_lfe_history_u32_words[128],
companion_lfe_history_u32_words[128].

## policy

Fields are generation U64, gain_db_history_u32_words[64],
exponent_refresh_history_u16_words[30] and
normalized_anchor_u16_words[30].

SHA-256 covers the exact canonical JSON bytes. No source, provider, receipt,
owner name or generation outside the object is prepended. The owner name is
carried by the enclosing receipt. The vectors provide complete source0
before/after objects, canonical preimage hex, byte length and SHA-256.

This closes the test-seam projection hash definition. It does not prove actual
FFT/transform or real ADM production execution.
