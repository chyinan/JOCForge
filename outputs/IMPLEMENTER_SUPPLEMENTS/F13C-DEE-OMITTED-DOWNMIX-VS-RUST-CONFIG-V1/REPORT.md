# DEE omitted downmix input vs current Rust configuration

## Result

The current Rust full-file job builder emits an explicit `<downmix>` block with `preferred_downmix_mode=loro` and all four mix levels set to `-3`. The bridge then accepts only that exact preference/level tuple and maps it to BSI codes `2` and `4`; the frame writer sets `mixmdate` and writes those fields.

The hash-bound DEE/DME trace reports the preferred control inactive. In that state, the DME request builder omits `preferred_downmix_mode`; the four mix-level controls are also inactive. The native `dmixmod` validator treats the field as optional and skips value validation when absent, but that trace does not identify the runtime enum default or a final F13C consumer.

This is a concrete source-level configuration mismatch: Rust currently supplies fixed values where the observed DME path supplies no user value. It is **not yet proof** that `loro/-3` differs from the native effective default, because the constructor/default owner and final BSI consumer remain unbound. Do not ask the user to select a mode and do not infer the native default from UI label or enum order.

## Compatibility action

Keep the UI/job input inactive as omission/unspecified, and carry that optional state through configuration. Before selecting a concrete BSI value, close the native settings-object initializer and its first consumer, or verify a directly selected DEE job's generated normalized job/profile that is tied to this run. Do not convert a display label to an encoder value implicitly. Whether the conditional BSI presence bit should be zero or whether DEE applies an internal default is still a native writer-contract question.

## Source identities

- Current JOCForge source snapshot at review: commit `8e558b868085c6b2c9a37a3cf1413c2bb8551dbb`.
- `encode_request.rs` SHA-256 `bf3f550899f12dbec2cc71c56ceeebc5d92659b53aba1d887afd5ccc74bb463e`.
- `codec_source_bridge.rs` SHA-256 `b3a5a8e1be761bb7dab62bfd4d3cc75cd3780f0d50f771d620e6189271c4b12e`.
- `frame_builder.rs` SHA-256 `170e3f220a795abfff5d296e8b53a8436b621e547c064e3aa3d4a781217c3829`.
- DEE source package `F13C-dmixmod-omission-default-trace-addendum-v21`, MANIFEST SHA-256 `cfaec056f570eec6d2a1d985bd3544df30eac446f89140ace9fa551448703415`.

No encoder run or media/PCM/payload/key read was performed for this comparison. The package does not establish the output sound or parser acceptance.
