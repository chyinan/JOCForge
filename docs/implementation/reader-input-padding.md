# Selected reader input tail padding

This provider helper covers one selected input shape: 38 channel rows, 1,536 samples per frame, and a final 1,392-sample read. It prepares fresh zeroed input rows, copies the opaque read prefix without changing any word bits, and leaves 144 positive-zero words at the end of every channel row.

The tail contains 5,472 F32 bit words, or 21,888 bytes. The helper operates on the reader/input side only. It does not accept renderer or DSP output rows, and no DSP output padding is inferred.

## Provider boundary

`SelectedReaderReadPrefix` distinguishes an input-reader prefix from other frame data. `prepare_selected_reader_input_frame` validates the selected channel and row lengths, initializes 38 rows of 1,536 positive-zero words, then copies each 1,392-word prefix into its row. The helper treats words as opaque bit patterns; it does not validate or numerically compare the valid prefix.

The native capture observed 1,747 positive-zero and 3,725 finite-nonzero words in the tail before clear. After clear, after short read, after interleave, at external API input, and at the four cached-input flush renderer consumers, 11 aggregate observations each classify all 5,472 tail words as positive zero. No PCM sample values or content digests were exported. The final packet reports 1,536 samples although the reader returns 1,392 actual samples; padding contents beyond positive-zero classification were not examined.

Tests use synthetic prefix words to check bit-preserving copy and positive-zero fill. Those inputs do not establish native valid-prefix numeric acceptance. The implementation is a local input-provider helper and is not evidence of full F13C or end-to-end encoder readiness.

## Scope limits

This rule covers one 38-channel, 48 kHz, 1,536-sample capture only. Other layouts, rates, malformed reads, general file seeking, PCM values, and product acceptance remain unverified. Do not apply this input rule to the renderer/DSP output tail, which the native observations classify as generally nonzero.

Wash handoff manifest SHA-256: `7e839dd2c4ff414651754056c40075360b1cb896422b926a64f4619b8af314f2`. Handoff check SHA-256: `d15b4f168cbb16d4041a8190a3898cf287f003aff2bcaf4bfc5c093c1313c768`. Wash replay reported 16/16 payload matches, 11 zero-tail observations, 0 mismatches, and `whole_F13C_ready=false`.
