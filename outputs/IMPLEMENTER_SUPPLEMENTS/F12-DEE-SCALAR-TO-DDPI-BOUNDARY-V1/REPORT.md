# F12 scalar-to-DDPi boundary

## Result

Static Ghidra evidence traces a native F32 scalar into an open-config block and through a normalizer. The normalized signed integer is used to compare against an internal byte and set a mismatch flag. The trace does not assign that result to DDPi ID `0x12` or to a BSI field.

The approved F12 model is separate: it takes F32 `selected_lufs`, produces integer DialNorm `[1,31]`, and defines the selected-profile U5 at bits 45–49. That model is not a native rule. The native scalar's unit is not stated, and the comparison between three measured scalars and model outputs remains conditional on the scalar being `selected_lufs`.

## Native path

In the hash-bound `dee_audio_filter_ddp_atmos.dll`, `FUN_180178dc0` reads an F32 at `EmpireEnc state+0x3c` and places its bits in the high 32 bits of open-config qword `+0x58`; the low word is `-120.0f`. `FUN_1801a7d50` passes the scalar to `FUN_1800fdc60`.

The helper returns `-120` for its near-sentinel branch. Otherwise it computes `CVTTSS2SI(F32(x-0.5))`, clamps to `[-31,-1]`, and returns that signed integer. The caller negates it only for a comparison with an internal byte; a mismatch sets flag `0x04`. No write to DDPi ID `0x12` is shown.

DDPi ID `0x12` is separately bounded by approved material as an integer at `stream+0x9c`, valid in `[1,31]`. The scalar/helper result-to-ID12 edge, the same-owner/generation join, and ID12-to-the-F12 BSI U5 edge remain `UNBOUND`. Do not turn equality of arithmetic results into an identity join. In particular, native `-120` sentinel return does not itself implement the model's U5 value `31`.

## Input-only examples

The vectors exercise only the recovered helper arithmetic. They do not establish the scalar unit, native measurement identity, or a BSI write. No PCM, payload, bitstream, key, or license material was read, and no DEE run was performed.
