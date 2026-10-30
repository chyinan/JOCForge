# Static drop-frame timecode contract; native vector not run

## Static rule only

The active parser's drop-frame marker is exactly a trailing lowercase `df` on a colon-delimited timecode. This static rule accepts the `29.97` rate token only, uses nominal 30-frame labels with a 1001/1000 scale, and removes two frame labels per minute except each tenth minute. With the marker present, labels 00 and 01 at second 00 are rejected on non-tenth minutes. The colon end frame is a time position; no numeric-token `+1` applies.

For the static example `00:01:00:02df` through `00:01:00:12df`, the corrected nominal frame indices are 1800 and 1810. The predicted file-position range is `[2,882,880, 2,898,896)`, 16,016 samples. This is a static arithmetic prediction only.

## Native and implementation status

The attempted native reference launch failed during filter initialization before workflow construction and did not parse the `df` input or create EC-3. Preserve `NATIVE_DF_CAPTURE=NOT_RUN` and `IMPLEMENTATION_READY=NO_FOR_THIS_VECTOR`. Do not use this prediction as a native fixture or approved implementation vector. A successful run with the same supported route and only this syntax change is needed first.

Semicolon punctuation, uppercase `DF`, other rates, embedded-timecode colon strings, and model output/comparison remain unevaluated. No source binary, decompilation, raw audio, raw logs or license material is included.
