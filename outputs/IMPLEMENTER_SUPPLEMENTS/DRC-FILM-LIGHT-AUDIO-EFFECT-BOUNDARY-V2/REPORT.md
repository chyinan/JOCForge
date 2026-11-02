# Film Light DRC profile to encoder audio-effect boundary

## Static rule

The authorized job asks for `line_mode_drc_profile=film_light`. The Atmos DLL's CRT initializer statically maps that profile-tree entry to U32 `0`. `FUN_1801535b0` returns this value in a property/status result at `param_2+0xa0`.

The encoder path is separate: `FUN_180178dc0` reads a raw Line enum from `param_1[1]`, and `FUN_1801aa730` writes `param_1[5]` to the DDP_COMPR destination `param_3+0x28`. The bounded Ghidra trace does not bind the profile snapshot U32 to that encoder enum. So the profile value is known, but its effect on the encoded DRC word/audio is not.

## Implementation meaning

This is not evidence that `film_light` is ignored, and not evidence that it changes decoded sound. Do not hard-code a no-op or claim exact DRC behavior from the tree value alone. The smallest useful next test is a matched DEE A/B on one authorized ADM (`film_light` vs `none`) and a decode-only PCM comparison; repeat on another ADM only if the first result supports a rule. Do not use output byte hashes as a proxy for audio difference.

## Scope

Wash independently rehashed the corrected source package manifest, seal, unique payloads, and selected Ghidra evidence logs plus the plugin image. The source package's 46-file/source list was not fully rehashed in this supplement. Ghidra 12.2 static only; DEE and PCM comparison were not run; no key/license material was read.
