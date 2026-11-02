# AtmosMezz auto-time normalizer to host-map boundary

## Wash disposition

The upstream Ghidra 12.2 receipt closes the typed normalizer call and its host-side map write. In this path, an `auto` policy resolves from an already present `offset` or `ffoa` property in the input metadata map; the receipt does not establish where those source map values were originally produced or transferred to the converter's `PrivContext`.

## Typed normalization rule

- `AtmosMezz::Run` calls `FUN_14012ff20` at RVA `0xf10da` with the host-local metadata property list and the input object's timecode-frame-rate, FFOA-policy, and offset-policy fields.
- In the `offset=auto` branch, the normalizer reads the existing metadata-map key `offset` and parses its value as a numeric double.
- In the `ffoa=auto` branch, it reads the existing `ffoa` and `offset` values and computes `metadata_ffoa - metadata_offset + normalized_offset`.
- The normalized entries are written back to the host-local property list and then stored by `FUN_1400f1cc0` at `dee::input::AtmosMezz+0x130`.

## Remaining transfer edge

The converter-side `FUN_18007fba0` reads `offset` and `first_frame_of_action` from `PrivContext+0xa8` in a separate DLL. There is no typed transfer from the host object's `+0x130` map to that converter map or same owner/generation join in the scoped evidence. Keep that cross-module relation `UNBOUND`; do not infer it from matching key names. The default file_position branch is available separately, but the originating source metadata-map values for `auto` remain unbound.
