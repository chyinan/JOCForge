# DEE LFE stride selection and transform-input boundary

## Wash disposition

The upstream Ghidra 12.2 package closes the native LFE branch selector field and the stage that supplies its 256-F32 transform buffer. It still does not establish the selected stride value for the target Atmos JOC job or join the native buffer to Rust's folded F08 input.

## Native selector and input construction

- `FUN_140317600` copies six descriptor pointers and their U16 strides into state arrays at `+0x12F0` and `+0x1320`.
- `FUN_140343530` / `FUN_1403448d0` choose a descriptor slot from `layout_id*6 + internal_lane`; when an LFE lane is present, `internal_lane` is the FBW count.
- The chosen descriptor's U16 `+0x0C` stride becomes arg1 to the LFE main/companion wrappers and their shared transform. Zero stride selects the cosine/sine recurrence; nonzero selects the two-table path.
- `FUN_14036e350` gathers 256 strided F32 values from the chosen descriptor into a temporary at `state+0x262E8`. The LFE wrappers pass that same temporary through the stride transform and recurrence finalizer. It is not the original BWF row pointer.

## Branch and comparison boundary

The matched job receipt identifies `atmos_mezz → encode_to_atmos_ddp` at 1024 kbps, but has no selected core layout, sample rate, or descriptor-stride value. Thus the selected stride branch remains `UNBOUND`; 5.1 and output-channel index 3 do not select it. The zero-stride formula is a match candidate for Rust `long_pre_rotation`; the nonzero path has no direct long-pre-rotation formula match. Neither is an audio-parity result.

The minimal native observation is a same-run receipt of descriptor pointer tokens, U16 stride values, selected layout/FBW count/LFE lane and slot, plus the temporary pointer identity at gather and transform entry. No buffer values need be captured. No DEE run, PCM, media, key, or license access occurred.
