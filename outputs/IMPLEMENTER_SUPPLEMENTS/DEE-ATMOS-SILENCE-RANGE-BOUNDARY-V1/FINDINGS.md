# Sample-range and DDP silence token rule matrix

The reused DDP schema declares `prepend_silence_duration` and `append_silence_duration` as strings defaulting to `0.0`. Decimal tokens are seconds; the parser multiplies by the source sample rate and casts to an integer, truncating toward zero. A lowercase `f` form accepts nonnegative integer frame counts and requires a numeric frame rate. At 48 kHz, the existing static vector `0.00390625` seconds produces 187.5 and returns 187. This is parser arithmetic, not a native output observation.

The two counts occupy separate per-source vectors and record slots. The first typed metadata consumer, `FUN_1801844c0`, copies prepend to composite row `+0x20` and append to `+0x38`. The next row copy does not read those fields. `FUN_180185240` forwards the row span to `FUN_180173c70`; sealed receipts show that helper only cleans the row string at `+0x18`. The subsequent vector release is not a sample consumer. The first actual sample consumer remains `UNBOUND`.

The intended `prepend → source span → append` order and `prepend + source count + append` length are implementation inferences from names and independent field flow. No native PCM placement, output-length delta, PTS change, or sample-consumer access was observed. Existing 36-channel zero controls establish only baseline configuration; no nonzero effect was isolated. No 38-channel source identity appears in the reused silence inputs.

The reused range receipts keep `start/end` separate from silence counts. `Priv+0xf98/+0xfb0` and the corresponding Reader `CompositeMasterItem.startSample/endSample` are sample-index fields; the Reader setup uses `endSample - startSample` as a sample count. The parent `0..47` file-position vector describes input frames, but its XML endpoint-to-sample convention is not established by execution.

A minimal prepend-only versus append-only contrast is recorded as `PROPOSED_NOT_RUN`. This task performed no DEE launch. No PCM, compressed payload, key, or license content was included or read.
