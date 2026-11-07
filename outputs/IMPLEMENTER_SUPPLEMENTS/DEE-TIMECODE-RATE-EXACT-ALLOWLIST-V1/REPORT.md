# timecode_frame_rate exact token allowlist

Reference-only Wash supplement. It binds the literal token set in one validator, not XML owner wiring or rate-conversion effect. `IMPLEMENTATION_READY=NO` for runtime timecode behavior.

`FUN_140135d90` returns 1 only for exact byte-and-length matches to: `not_indicated`, `23.976`, `24`, `25`, `29.97`, `30`, `48`, `50`, `59.94`, `60`. Other strings return 0 in this function. The direct token `29.97df` is rejected here; the sealed parent shows `FUN_14012ff20` canonicalizes it to `29.97` and writes a separate `drop_frame` property before validation.

This validator does no numeric conversion, unit calculation, or rounding. Parent conversion helper semantics and XML owner/generation remain `UNBOUND`; runtime remains `NOT_RUN`.