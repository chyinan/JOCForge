# Attempt-10 downmix values: template origin vs user input

## Result

The current JOCForge `selected_full_file_job_xml` template writes `preferred_downmix_mode=loro` and all four mix levels `-3` as literals. The function takes only input/output file-reference parts; these values are not parsed from a user CLI option. The selected configuration parser requires the downmix node and all five values and rejects missing fields. Thus attempt-10's reconstructed JOCForge job has a program-generated explicit downmix configuration, not a user-selected value.

This is separate from DME's UI request builder: its `PreferredStereoDownmix` control is inactive and omitted, and inactive mix-level controls are also omitted. That UI omission does not prove the JOCForge CLI path omits values. Conversely, the Rust template's `loro/-3` is not proven to be DEE's native effective default or to match the protected DEE output's downmix profile.

The v22 package reconstructs the attempt-10 XML SHA from the current source template and run-bound input/output paths; the application did not capture the runtime XML digest. Its source commit postdates the attempt-10 build, and the run receipt binds executable/runner hashes, not the source commit. Keep the exact attempt-10 template-to-runtime link at `RECONSTRUCTED_NOT_RUN_CAPTURED`.

## Implementation consequence

No user prompt is indicated. Preserve the current source-generated values if reproducing the current JOCForge attempt-10 profile; do not claim they are the DEE default or derive them from the separate JOC profile label. To change them for DEE parity, first bind the selected DEE job's actual profile or a direct native enum initializer/consumer. The native omitted-field default and final F13C/BSI effect remain unbound.

## Hash-bound evidence

The v22 package's eight SHA256SUMS entries were independently recomputed and matched; the reconstructed XML parsed as `job_config`. No audio output, WAV, PCM, private key, HMAC secret, or license material was read.
