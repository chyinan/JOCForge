# R138 pre-Run reader observation with input override

## Observation

R2 recovered the exact hash-bound R138 profile XML and used the authorized “人是猫” input bytes through a task-local ASCII alias. WinDbg reached the host virtual `Run` callsite `dee.exe+0xCA9FC` and stopped before the call. It captured the wrapper, reader-owner, vtable, slot-6 target, thread, and runtime module identities. The launched process was terminated while stopped; `Run` was not called and no output was created.

## Type boundary

The sanitized profile receipt declares `input_type=audio.adm`. The command also supplied `-a` with the authorized ADM BWF alias. The live reader vtable maps in Ghidra to `dee::input::AtmosMezz`, slot 6 targeting `FUN_1400F09D0`. Because the profile input was overridden, this observation does not tell whether the runtime reader is the selected `audio.adm` factory or the `-a` override path. Preserve the result as `PROFILE_DECLARATION_VS_RUNTIME_READER_UNBOUND`; do not call it a confirmed R138 selected reader or a proven profile mismatch.

At the observed generic getter callsite, six `audio_filter` getters were seen; no `audio_input` getter was captured there. Getter table/count/local callback slot, StageRunner candidate index, and the selected reader's typed owner/generation remain `UNBOUND`.

## Disposition

This package closes only a same-invocation pre-Run boundary for an overridden-input launch. It does not close the R138 selected reader. The next discriminating observation is a pre-Run capture with the exact profile and no `-a` override, using only the same hash-verified authorized ADM bytes if that profile input path can be satisfied without altering other profile fields. If not, retain `NOT_RUN` for that comparison. Do not step into `Run`, read media bytes, or start a full encode for this observation.

No PCM, row/table bytes, payload, key, or license contents were read. The sample was SHA-256 checked for identity only. This is reference-only and adds no audio algorithm rule.
