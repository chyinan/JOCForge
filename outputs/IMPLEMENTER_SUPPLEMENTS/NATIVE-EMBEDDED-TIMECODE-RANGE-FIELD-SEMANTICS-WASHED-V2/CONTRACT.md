# Embedded-timecode range field semantics

## Static field contract

Hash-matched Ghidra analysis binds the native range descriptor fields to StartSample and EndSample. Both are sample-boundary values, not byte offsets. The selected path substitutes AtmosStorage.sample_count when EndSample is zero, rejects EndSample greater than sample_count, and rejects StartSample greater than EndSample.

The native field names and validations do not state whether EndSample is inclusive or exclusive. The endpoint convention and the absolute sample-position origin remain `UNBOUND`.

## Run-121 dynamic correlation

Run-121 used the authorized ADM input and the verified D-drive DEE executable. In the same run, the native range record reported start 0/end 49,999; each reader read 49,999 sample frames per channel (32 × 1,536 plus 847), while the final callback descriptor still spanned 1,536 frames. Pre-read zero-fill accounts for the remaining 689 elements per channel on the tail callback. These are separate values and domains; numeric agreement does not decide EndSample endpoint semantics.

Run-115 remains unchanged and is a separate execution. The R115-to-R121 owner/generation join is `UNBOUND`. The 24 fps / 2,000 samples-per-frame relation remains an inference; XML does not specify timecode_rate. Sample position to spatial callback generation remains `UNBOUND`.

No PCM or EC3 payload is included. Model execution and uninstrumented-output comparison remain `NOT_RUN`.
