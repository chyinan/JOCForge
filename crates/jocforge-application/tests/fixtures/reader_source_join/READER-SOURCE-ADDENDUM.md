# Selected reader and source-position join

For selected item index 0 at 48 kHz, the observed CompositeMaster fields are: 38 channels, start sample 2,880,000, end sample 2,886,000, duration 6,000 samples, prepend/append 0, frame size 1,536 samples. The four external API calls carry selected-segment-relative control positions 0, 1,536, 3,072 and 4,608. Each is joined to the same selected reader/master tokens, cursor-before value, read_float request/actual count, returned packet, control table and interleave-source table.

For this selected capture only:

```text
source_sample_start = selected_item_start_sample + reader_cursor_before_samples
```

The resulting selected source sample starts are 2,880,000, 2,881,536, 2,883,072 and 2,884,608. All 152 control records (38 channels × 4 calls) carry the matching relative position. `read_float` requests 1,536 samples each call and returns actual counts 1,536, 1,536, 1,536 and 1,392 with status 0. The returned packet reports 38 channels × 1,536 samples even on the final call. Padding contents were not observed and must not be guessed.

This binds selected source-sample coordinates, not absolute file byte offsets or host wall-clock time. The constructor item and CompositeMaster descriptor are matched by selected index and exact field values; pointer identity is not proven. No PCM value or PCM content hash was read or retained. One selected item/job is covered; seek, multiple items, resampling, malformed input, and same-owner reset are not covered.
