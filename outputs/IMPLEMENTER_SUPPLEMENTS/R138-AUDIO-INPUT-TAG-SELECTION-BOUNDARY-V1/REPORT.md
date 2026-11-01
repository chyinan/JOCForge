# R138 audio-input tag selection boundary

## Static host rule

For the hash-bound host image, `FUN_140049120` enumerates registered audio-input configs into StageRunner vector `+0x2F0`. `FUN_140042610` constructs the `audio.input.type` schema from eligible candidates; that schema loop is not runtime selection.

The R138 profile receipt says input type `audio.adm`, while `encode_to_atmos_ddp` is a separate output-filter route. The host stores the XML child tag at `StageObjectWithStorage+0x210`; its selector scans a distinct candidate vector at `+0xB0..+0xB8`, asks each candidate's virtual `+0x10` getter for a string, and compares equal length plus bytes. Each match replaces `+0x1F8` and scanning continues, so the last equal-name entry wins. A missing match or selected config with `+0x0C == 0` fails before the factory helper result is stored at `+0x1F0`.

## Open identity join

The selector's `+0xB0` vector is not joined by typed identity to the StageRunner `+0x2F0` vector. The successful R138 receipts do not record effective module order, getter values, duplicate-name outcome, or selected index. Therefore `audio.adm` plus the output route is insufficient to derive the R138 selected candidate index.

## Disposition

Reference only; `implementation_ready=false`. The exact tag-match rule is statically bounded, but keep the selected StageRunner index and reader owner `UNBOUND`. The next minimum field is `selected_stage_runner_input_candidate_index` captured at the successful `adm` match and resolved against the same-run `+0x2F0` vector; emit `NOT_IN_VECTOR` if the pointer identity join fails. That observation remains `NOT_RUN`.

No DEE run or media read occurred for this addendum. It does not add an audio algorithm rule or authorize implementation behavior.
