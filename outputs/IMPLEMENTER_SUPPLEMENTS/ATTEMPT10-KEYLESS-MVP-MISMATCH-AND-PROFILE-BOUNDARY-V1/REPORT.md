# Attempt-10 keyless MVP mismatch and profile boundary

## Result

Attempt `sample6-keyless-full-20261101-10` completed with application `DONE` and exit code 0. Its run receipt binds the authorized “人是猫” input SHA-256 `7826dcbef37ef69642a6164c97a37bcfb3f917cea74aa0f0a30d76d0a825f1d1`. The output and user-designated keyless comparator have equal lengths, 17,395,712 bytes, but different streaming SHA-256 values:

- Attempt-10 output: `593b6f866bf4396d25cd7b6307515f965f69e9884ce46599675c6f5d0a8c0473`
- Keyless comparator: `383d7de4a57fd9bb32b8f871af1d2b5b0ad686a4b25e5fc6e51cd1be3adf6e86`

The user's exact-hash MVP criterion therefore fails for this attempt. The comparator is user-designated as the keyless Python prototype output; the input ADM identity is user-confirmed and crosswalk-bound. A sealed Python run/source receipt remains absent; that provenance caveat does not block the byte comparison.

## Target-profile evidence

Run-scoped receipts and logs bind the run, input, executable, runner, successful application status, output length, and output hash. They do not bind output sample rate, output bitrate, output channel layout, E-AC-3/JOC profile, or a same-attempt job XML. The prepared-input record says 48 kHz / 38-channel PCM24; that describes the input and does not establish the output profile. Thus the measured hash mismatch is real for this run, while whether the run used the requested 48 kHz / 1024 kbps / 5.1 E-AC-3 JOC output settings remains `UNBOUND`.

## Terminal receipt boundary

The run-scoped logs and exit marker report `DONE` and exit 0. The persisted result's `application_exposure_receipt` is marker text, not a structured final-exposure receipt with explicit parent-fsync fields. Keep the evidence at the application-level terminal status; do not invent serialized receipt fields.

The comparison read only run receipts, file metadata, and streaming SHA-256 results. Neither EC-3 file was decoded, parsed, rendered, or played. No cause is inferred from the mismatch. Render and playback remain `NOT_RUN`.
