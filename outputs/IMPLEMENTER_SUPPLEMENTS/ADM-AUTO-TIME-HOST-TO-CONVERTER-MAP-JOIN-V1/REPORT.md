# AtmosMezz host-normalized time map to converter context boundary

## Wash disposition

The upstream R3 Ghidra receipt closes the host-side typed writer: normalized `offset` / `ffoa` properties are written into the host `AtmosMezz+0x130` map. It does not show a typed transfer from that map to the converter DLL's `PrivContext+0xa8` map.

## Typed host writer

In `dee.exe`, `AtmosMezz::Run` passes the existing metadata property list plus the input object's timecode-frame-rate, FFOA policy, and offset policy fields into `FUN_14012ff20`. The normalized entries are then stored by `FUN_1400f1cc0` at the host input object `dee::input::AtmosMezz+0x130`.

The separate converter consumer is `FUN_18007fba0` in `dee_audio_filter_convert_atmos_mezz.dll`; it reads `offset` and `first_frame_of_action` from `PrivContext+0xa8`.

## Remaining edge and MVP boundary

No call argument, store, copy, or constructor edge connects host `AtmosMezz+0x130` to converter `PrivContext+0xa8`, and no same owner/generation join is established. The exact first missing edge is that typed transfer. Default `file_position` MVP remains unblocked; auto-time extension remains `UNBOUND`. No DEE run or PCM/payload/key/license access occurred.
