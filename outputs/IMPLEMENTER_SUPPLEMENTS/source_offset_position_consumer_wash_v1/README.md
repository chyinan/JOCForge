# Source offset and first range consumer

This Wash-only addendum binds the seven previously authorized ADM identities to a metadata-only RIFF scan, then records the typed source-time adapter and the first explicit file-position range consumer.

All seven verified inputs carry the same ADM `audioProgramme.start` token, `01:00:10.00000` (3,610 seconds). Their metadata has no BEXT time reference and no explicit XML `offset` or `ffoa` field. The scanner seeks over every audio `data` chunk and reads zero audio bytes. The R138 native log for the one 38-channel input reports `ffoa=offset=3610`; that equal pair does not discriminate precedence. Six 36-channel samples were not admitted to this selected 38-channel path, so their native pairs remain unobserved.

Static analysis binds two source master metadata values as binary64 seconds, with one normalized to `offset` and the other to `ffoa`. The input schema declares `ffoa=auto` as retaining FFOA from the source. The file-position range consumer uses `max(offset, ffoa) - offset` for the `first_frame_of_action` start token; its multi-file path computes a rounded `(ffoa - offset) × sample_rate` sample adjustment. The source master virtual field writer remains unresolved.

This range-origin edge is a must-have for selecting the exact samples that reach the codec. The first consumer that carries an absolute sample position into spatial/fit was not observed. Host frame, spatial callback, fit generation, U04 invocation, and consumer generation remain unbound. Model execution, Type10/six-plane composition, and Gate 1 closure were not performed by this addendum.

Verify the package with:

```text
python verify_source_offset_position_consumer.py
python -O verify_source_offset_position_consumer.py
```
