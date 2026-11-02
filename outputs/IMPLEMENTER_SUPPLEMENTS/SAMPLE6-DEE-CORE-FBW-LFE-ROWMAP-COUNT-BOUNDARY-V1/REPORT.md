# Sample 6 DEE core FBW/LFE row-map count boundary

Recipient: implementation evidence supplement  
Disposition: static shape and sentinel behavior are bounded; count semantics and 36/38 admission remain unbound.

## Checked static shape

For the hash-bound `dee_ddp_encoder.exe` image (`0cf23520799f52323b0994e5fcc16924da7b027a5b83a0cff3b4f06792c18532`), `FUN_140317600` copies six fixed 16-byte stream descriptors into six-entry pointer/source-slot arrays. `FUN_140343530` loops over six channels and uses a dynamic selector-position count from `context+0x19132`. It reads a six-wide layout-table row, then uses a signed source-slot value to select descriptor samples.

The row-map builder `FUN_140329a10` has six outer rows at the checked callsite and a five-slot row stride. `context+0x19130` controls the slot loop and a branch threshold. The functions in scope do not clamp this count to five. A count greater than five would index into a following row under this layout, but the semantic producer, backing extent, and runtime behavior are not bound.

Trailing layout value `0xffff` sign-extends to `-1`; no sentinel check appears before indexing the six-entry descriptor arrays. The count of layout positions is not locally clamped to six. Whether the selected producer always excludes sentinels and constrains the selector remains `UNBOUND`.

## Relation to arbitrary ADM track count

No 36/38 comparison occurs in the checked row-map path. This package does not prove that source-slot is a zero-based physical BWF row, that the model count equals this row-map slot count, or that all ADM rows are consumed. An input-only discriminator with `source_slot=36` is conditional on a future typed row identity and is not an acceptance result. Preserve the user-confirmed decoded output channel 3 LFE identity separately from input-row mapping.

No DEE run, debugger attachment, PCM/payload/key read, or runtime capture occurred. The upstream package has no checker script; Wash independently verified its manifest, seal, sums, and four payload hashes/sizes.
