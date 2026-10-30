# End-of-file sentinel to source-reader extent

## Bounded native rule

For the selected 38-channel ADM BWF fixture, `time_base=file_position`, numeric start token 3240 at 24 fps, and end token `end_of_file`, the native composite record preserves `end=0` as an EOF sentinel. In the observed active source-reader path, the end getter resolves the physical source extent to 6,522,906 samples; with cursor 6,521,472, the remaining request and return are both 1,434 samples and the cursor advances to 6,522,906.

The selected source interval is `[6,480,000, 6,522,906)`, 42,906 samples / 0.893875 s. The encoded EC-3 track is 0.896 s; keep the clocks separate. This confirms the sentinel reaches physical EOF in this reader path, but it does not rewrite the logged composite end field from zero or define how an encoder fills the final incomplete frame.

## Limits

This is one successful file-position reader vector. Embedded-timecode EOF, other readers, zero-length/reversed intervals, multi-row and append/prepend, final-frame fill semantics, model output and model/native comparison remain unverified. No raw samples, PCM, source code, raw pointers or binary artifacts are included.
