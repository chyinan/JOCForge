# Sample 6 DialNorm receipt scalar gap

The full current JOCForge run is bound to the authorized input hash and output
hash. It logged Rust meter DialNorm 17 and emitted U5=17 at bit 45/width 5.
The production source commit contains the selected-host-word F32 formula
path, but the exact `selected_lufs`/`selected_host_word` numeric input is
absent from the run receipt and logs.

The same-input DEE U5=21 observation belongs to a bounded file-position window
2160–2208, not a full-file output. It cannot be labeled a same-scope mismatch
against the Rust full-file first AU. A full-file DEE U5 and audio comparison
remain NOT_RUN. The minimal nonsecret receipt field for the next run is
`rust_selected_lufs_f32_word`, captured at the formula input and bound to the
run ID, input, executable and source commit; this allows exact formula replay
without PCM or payload.

This package makes no claim that the formula is correct or incorrect for the
current run. No new DEE run, payload read, old comparison-group use, or private
key access occurred.
