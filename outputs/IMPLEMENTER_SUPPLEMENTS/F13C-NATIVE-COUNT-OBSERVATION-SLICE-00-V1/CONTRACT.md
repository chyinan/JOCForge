# Native count observation: 0..48 frame slice

Classification: NATIVE_OBSERVATION_ONLY; informational for internal implementation review.

A bounded native run recorded 64 U32 values at a count-output store over file-position frames [0,48) of the identified 38-channel, 48 kHz, 24-bit ADM BWF. The observed field is named counted_body_bytes in the source payload; this package does not establish what serialized object those values count. Values range from 239 to 331 and all recorded stream selectors are 0.

The selected job metadata in the source records 1024 kbps, measurement mode 1770-4, DI disabled, speech threshold 100, line and RF DRC none. Codec/JOC mode and an AU identity are not recorded by this slice. It therefore supplies no E-AC-3/JOC payload rule, frame-size formula, target-length rule, or implementation requirement. The values must not be used as clean-room expected outputs or an independent oracle, nor as proof of an AU join.

Only the slice-local absence of 507/508/511/512 is reported. It cannot be generalized to later frames or to the full programme. Same-parent generation, serializer, refill, and selected F12/AU joins remain NOT_CAPTURED/UNBOUND.

No PCM samples, EC-3 bytes, license contents, source log, binary, disassembly, address, or source code are included. The capture method and source artifact identities are represented only by the hash-bound provenance record. Wash verified payload hashes and the numeric vector's internal count/range/selector consistency; it did not repeat the native capture or independently validate the sender-reported output-control run.
