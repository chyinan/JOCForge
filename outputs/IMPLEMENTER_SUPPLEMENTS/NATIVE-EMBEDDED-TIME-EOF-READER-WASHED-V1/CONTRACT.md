# Embedded-timecode EOF reader vector

## Bounded native rule

For the selected 38-channel ADM BWF fixture, numeric embedded-timecode start token 89880 at 24 fps and transcribed embedded origin 3610 seconds normalize to file position 135 seconds / sample 6,480,000. With `end=end_of_file`, the composite log preserves `end=0` as a sentinel. The observed active reader resolves the physical source end to 6,522,906; with cursor 6,521,472, it requests/returns 1,434 samples and advances to 6,522,906.

The available source interval is 42,906 samples / 0.893875 seconds. The encoded EC-3 track is 0.896 seconds; do not conflate these durations. The final incomplete-frame fill behavior is not evaluated.

## Evidence limits

The start normalization is based on 24-fps and 3610-second origin metadata transcribed from run 43; its raw debugger transcript was not persisted. The run-60 reader cursor/end/request/return scalars are direct hash-bound capture references, but the raw transcript is excluded and not rehashed by Wash. This establishes one embedded-timecode EOF reader vector only. Other origins/rates, alternate syntax, multi-row, final-frame fill and model comparison remain unverified.
