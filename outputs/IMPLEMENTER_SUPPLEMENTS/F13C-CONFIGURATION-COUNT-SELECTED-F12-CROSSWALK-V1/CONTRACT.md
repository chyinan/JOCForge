# Configuration count and selected F12 complexity are separate paths

Status: `CAPTURE_LOCAL_PRODUCER_CLOSED; SELECTED_F12_MODEL_RULE_EXPLICIT; NATIVE_CROSS_PATH_UNBOUND`.

The selected F12 model obtains its E-AC-3 JOC `complexity` byte from the same-source validated EMDF: `U8(ID14.object_count + 1)`. In the selected model example, ID14 `object_count=15` yields 16; sources 0 and 1 use absent published payload and do not write this JOC field.

A separate bounded capture-local configuration record path reports `rate_code=5`, 38 signed-I16 channel labels, 15 non-LFE channels and LFE-present=1. Its count is signed I32; parameter 13 queried with selector 10 returns 16 and reaches a BSI producer/importer field `complexity_index_type_a` at frame bit 105. That capture-local value has no source or AU generation tags in the reviewed material.

The numeric value 16 and 8-bit widths do not establish an identity join. Do not feed the capture-local configuration count to the selected F12 writer. Keep parameter 13/selector 10, F01E profile pair key 13, ID14 downmix_index, ID14 object_count, ID11 slot_count, carrier_block and transport first_block as distinct domains. Native captured-record→selected F12 source/frame/AU identity remains `UNBOUND`; the model-side EMDF producer rule is explicit and implementable.

The complete bounded field map, three input-only cases, and source identities are in the accompanying JSON files. They contain no expected DSP output. Native execution, per-AU captured count use, native selected wrapper mapping and full-song replay remain `NOT_RUN`.
