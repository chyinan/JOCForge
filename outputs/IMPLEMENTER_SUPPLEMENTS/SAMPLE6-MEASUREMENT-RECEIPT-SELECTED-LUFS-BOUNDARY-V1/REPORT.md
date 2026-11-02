# Sample 6 selected-LUFS run-receipt serialization boundary

`ProgrammeMeasurementReceipt` already contains `selected_lufs_word` as a
`u32` F32 bit pattern together with `meter_terminal_sha256` and programme,
attempt, input and config identities. Construction passes
`measurement.selected_word`; the same receipt remains available through
`ProgrammeOwnerRoots` and `SourceExecution`. The receipt struct does not
derive `Serialize`, so this field is not automatically emitted by the managed
run JSON adapter.

For the recorded full Rust run, the JSON has `rust_logged_meter_dialnorm=17`
but no selected F32 word or meter terminal hash. The run's source commit
`216326f...` also did not log the word. Current source can read and print the
word in `encode_request.rs`, but the managed adapter remains the final
serialization boundary and must expose it explicitly for replay.

The minimal diagnostic change is one non-sensitive JSON field
`measurement_receipt.selected_lufs_word_hex`, sourced directly from the
existing `ProgrammeMeasurementReceipt.selected_lufs_word()`, with its
`meter_terminal_sha256` and existing programme/attempt/input/config identity
in the same receipt. Do not infer the F32 word from U5, header bits, or a
canonical digest. This is observability only; it does not change the audio
algorithm or justify a new U5 formula.

No new DEE run or media/PCM/payload/key access occurred. The old DEE scalar
`-20.665958` remains a bounded file-position observation and is not the
missing Rust input word.
