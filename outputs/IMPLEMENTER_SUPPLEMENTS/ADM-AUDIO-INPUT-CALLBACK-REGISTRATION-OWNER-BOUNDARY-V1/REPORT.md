# ADM audio-input callback registration boundary

## Result

Static Ghidra evidence binds the host's `audio_input` getter result to a raw callback-pointer vector. It does not bind a callback entry to an `AdmPlugin` typed owner, a selected runtime factory index, or the already observed AtmosMezz reader instance.

## Getter and registration path

`FUN_14016AF20` maps the category string `audio_input` to export `dlb_phoenix_get_audio_input_handle`. `FUN_14016A1C0` resolves and calls the getter. The returned record contains a function-table pointer at `+0` and a signed 32-bit entry count at `+8`; entries are read at 8-byte stride.

At RVA `0x16aa9e`, each raw callback pointer is passed in `R8`, category in `RDX`, and PluginManager owner in `RCX` to `FUN_1401677f0`. That function appends callback pointers to the vector at owner `+0x18`; `audio_input` and `video_input` share this storage. The entries do not retain an input-plugin type label or an `AdmPlugin` owner token.

## Index-domain separation

The static StageRunner initializer position 6 is a separate 16-byte function/context descriptor. It creates an `AdmPlugin`. The `AdmPlugin` vtable slot 6 separately creates an AtmosMezz reader. The getter's runtime table index is a third domain. No static conversion joins these indices; equal numeric values do not establish identity.

## Runtime boundary

The prior derived First Love observation reached the AtmosMezz reader before its `Run` call. It did not capture the getter's returned table/count, selected callback pointer, owner, or same-task generation. No new process was run. The selected callback-to-AdmPlugin-to-reader join remains `UNBOUND`.

The minimum next observation is a bounded read-only capture of the getter result and registration call, then a one-shot callback observation correlated with the existing pre-Run reader and same-task owner/generation. Stop before `Run`; do not infer runtime selection from initializer position.

No raw Ghidra trace/script, job XML, runtime pointers, PCM, bitstream, key, or license content is included.
