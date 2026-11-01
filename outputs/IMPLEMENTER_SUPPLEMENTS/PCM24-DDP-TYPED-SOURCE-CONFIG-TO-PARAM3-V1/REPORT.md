# Typed DDP source-option to per-source record boundary

## Scope

This addendum binds the typed configuration key `dee_audio_filter_source` to the DDP private-state source option, then binds its per-source map entry to the measurement-record writer argument within one DDP measurement activation. It does not bind the host-selected ADM object or its generation to that entry.

## Static typed path

`FilterDDPJocEnc` vtable slot 1 resolves to `FUN_18013E9D0`. It enumerates host `param_2` key/value entries, looks each key up in the private `ParametersContainer` at `private+0xEC0`, and invokes the registered property setter through vtable slot `+0x28`. The option `dee_audio_filter_source` is registered at `private+0xA0` as `SafeType<vector<map<string,string>>>`; `dee_audio_source_type` updates the source count at `private+0xF18`. The source maps carry filename and timing/format metadata, not an observed host object or generation pointer.

`FUN_180145380` builds the DDP per-source set from the input/output maps. `FUN_18014ABA0` selects an entry and passes it as `param_3` in R8 to `FUN_1801535B0`. The prior pointer-only capture observed the same DDP owner token at the map builder and writer entry in one `FUN_18014ABA0` measurement activation. This binds the DDP internal source-entry-to-record argument path, not host ADM identity.

## Exact remaining join

The host/framework writer of `param_2` into `FUN_18013E9D0` has not been traced to a typed host owner/generation field. The option's value type is only filename/metadata maps. Config-file hash, input-BWF hash, matching path strings, and a shared outer StageRunner frame do not establish a selected ADM object or source-generation identity. No new encoder run or media read is part of this addendum.
