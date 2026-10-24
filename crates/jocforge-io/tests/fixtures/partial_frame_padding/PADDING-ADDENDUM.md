# Selected partial-frame reader padding

For the selected 38-channel, 48 kHz, 1,536-sample reader frame, the input row table is cleared before every `read_float` call. On the final call the reader requests 1,536 samples and returns 1,392 actual samples. The returned packet still advertises 1,536 samples.

The unwritten tail is 144 samples per channel, 5,472 F32 words total (21,888 bytes). Immediately before the clear, the observed tail contained 1,747 positive-zero words and 3,725 finite nonzero words. After the whole-row clear, after the short read, after interleave, at external API input, and at each of the four cached-input flush renderer consumers, all 5,472 words are positive-zero bit patterns. The clear uses positive zero; do not substitute negative zero or NaN.

The existing frozen Python `full_host_frame` model also preserves the actual 1,392-sample prefix and appends 144 positive-zero F32 words. That establishes the input-tail rule for this selected path. It does not establish the contents of the valid prefix by this package. The renderer/DSP output's own 144-sample tail is generally nonzero; do not apply the reader padding rule to DSP output.

This package exports aggregate word classifications, shapes, pointer/row-table identity predicates, and comparison booleans only. It exports no PCM sample values or content digests. It covers one selected source item, one 48 kHz format and the observed flush consumers. Other sample rates, row layouts, malformed inputs and whole-programme acceptance remain unverified.
