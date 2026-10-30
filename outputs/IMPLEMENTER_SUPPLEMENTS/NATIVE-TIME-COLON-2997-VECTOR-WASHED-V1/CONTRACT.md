# Colon-delimited timecode vector at 29.97

## Bounded addition

For the selected 38-channel ADM BWF fixture, file-position time base, filter override 29.97, and non-drop colon form `HH:MM:SS:FF`, the native parser maps `00:01:30:00` and `00:01:30:10` to nominal frame positions 2700 and 2710. It applies nominal rate 30 with scale 1001/1000. The colon end frame is used directly as a time position; the numeric-token inclusive-end `+1` does not apply to this branch.

The resulting source interval is `[4,324,320, 4,340,336)`, 16,016 samples / 0.333666667 seconds. Both sample products are integral, so this vector does not test fractional quantization. The active writer truncation rule is independently supported by the 29.97 numeric-token dynamic capture in v4.

The encoded EC-3 track duration is 0.352 seconds and remains separate from the selected source interval. This closes only the specified colon-form, rate and file-position case. Semicolon/drop-frame, other colon rates, embedded-timecode colon strings, model output and model/native comparison remain untested.
