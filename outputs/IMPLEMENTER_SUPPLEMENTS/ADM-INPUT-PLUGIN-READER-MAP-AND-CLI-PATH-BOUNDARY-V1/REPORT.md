# ADM input plugin and CLI path boundary

## Result

Hash-bound static DEE evidence maps the input plugin type `adm` to the concrete `dee::input::AtmosMezz` reader. It also shows the traced `-a`/`--input-audio` option transfers paths into the job-config audio list without changing the reader factory/type on the inspected edges. This corrects an earlier profile-type mismatch interpretation.

## Static mapping

`dee::input::AdmPlugin` advertises type label `adm` and a BWAV-ADM description. Its factory vtable slot 6 targets `FUN_1400EF750`; the factory constructs a reader and calls `FUN_1400EDFA0`, which installs the `dee::input::AtmosMezz` vtable. The reader supports optional `file_name`. A distinct `AtmosMezzPlugin` wrapper uses the same concrete reader factory.

The static StageRunner initializer ordinal 6 constructs the AdmPlugin wrapper. It is not proof of a dynamically selected `audio_input` factory index.

## CLI path boundary

`-a`/`--input-audio` appends a path to the CLI audio list. The traced transfer copies that list to JobConfig owner `+0x7D0`; the inspected parser/transfer/property-handler edges contain no reader-type or factory-index write. WAV and AtmosMezz property handlers handle `dee_cmd_input_audio` as `file_name` on an already selected object. The final identity join from the JobConfig vector to the selected reader setter remains `UNBOUND`; therefore this does not prove global path-only behavior.

## Derived observation and limits

A derived First Love job changed only input/output/temp paths and retained the normalized nonpath digest. At a pre-Run breakpoint, its CLI and copied JobConfig audio vectors were empty and its reader vtable identified AtmosMezz. The reader Run call was not made; no output was created. This is a derived metadata-only pre-Run observation, not original R138 or human-cat audio acceptance.

Remaining joins include the dynamic `audio_input` getter table/count, selected factory index, typed owner/generation, final path-list-to-reader setter owner, and reader-output-to-first Atmos consumer. Do not infer them from static initializer ordinal, function/event order, or equal strings. The package contains no Ghidra trace, decompilation, code, job XML, raw pointer, media, payload, or key.
