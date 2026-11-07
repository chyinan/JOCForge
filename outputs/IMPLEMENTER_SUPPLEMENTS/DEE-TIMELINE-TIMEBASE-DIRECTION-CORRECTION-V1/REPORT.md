# DEE timeline time-base direction correction

## Correction: source-offset subtraction direction

This addendum corrects two earlier Wash supplements that reversed the `time_base` branch. In the hash-bound DDP Atmos plugin, `FUN_18014FD90` compares the `+0x550` time-base string with `embedded_timecode`; on exact match, that branch subtracts the source metadata `ffoa` offset from both start and end before storing file-relative sample boundaries. The nonmatching `file_position` branch does not subtract `ffoa`.

Therefore the prior rule “file_position subtracts offset; embedded_timecode does not” is superseded. The selected input's resolved `ffoa` value remains `UNBOUND`; XML `<offset>auto</offset>` is a separate field and does not identify that runtime value.

## Time parsing and sample math

- Dotted `HH:MM:SS.xx` form parses as seconds.
- Colon `HH:MM:SS:FF` form parses frames using the indicated timecode rate.
- An integer without separators follows a frame-count path; a nonempty integer `end` adds one frame before conversion. A zero end sentinel follows a separate source-duration path.
- If the configured `timecode_frame_rate` slot is absent, the parser searches source metadata for that key and installs it if present. The selected input's effective rate is `UNBOUND`.
- Validation uses `round(sample_rate * adjusted_seconds)`. Stored sample boundaries use a truncating integer cast of `adjusted_seconds * sample_rate`.
- Decimal silence duration is converted to samples with truncation; frame-form silence converts via frame-rate seconds and then truncates.

## Encode and tail boundary

The encode worker submits 1,536-sample frames and then zero-size flush frames until the output byte span is empty. The process wrapper accepts 1,536 samples or zero and returns a byte span and length; this does not establish one AU per result or total AU count.

An Agent 6 static receipt supports a conditional zero-tail inference when the AtmosReader branch is selected, a short read writes only its returned sample count, and the row buffer was pre-cleared. The link from requested prepend/append duration to that fill site and runtime reader selection remain `UNBOUND`/`NOT_RUN`.

## Supersession and scope

Supersede only the reversed `file_position`/`embedded_timecode` direction in `DEE-ATMOS-DDP-TIMELINE-XML-MAPPING-V1` and `DEE-TIMELINE-RANGE-TO-ENCODE-PASS-V1`. Other independently bound field parsing and count rules remain as stated in their packages. The vectors below are synthetic arithmetic only, not captured job values or output.

Source package `DEE-ATMOS-TIMEBASE-ENCODE-CORRECTION-WASH-V1` manifest SHA-256: `59426234b80bd3e6858d89f8a4edb958ea93fc5143fbf074b8ba029285db202e`. Wash verified all 18 payload hashes and the manifest/seal link. No DEE run, PCM/payload read, or key/license access occurred.
