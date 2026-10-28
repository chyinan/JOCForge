# Prefix diffusion ownership and bed control boundary

Grade: approved selected-profile specifications plus rehashed frozen model, with the limits below. No native pointer or observed byte claim is inferred.

## Prefix and diffusion state

F04A.4 and F04F.5 assign each active diffusion instance to a **regular tail**, indexed in the 37 non-LFE input tails. The F03W `render_audio` state schema has optional diffusion instances and previous outputs indexed by those 37 tail slots. Each active instance owns one independent four-lane filter state and four previous-output halves; the eleven prefix outputs select mode/lane from that tail's filter outputs when their branch requires it. The eleven prefix outputs are not eleven independently owned persistent diffusion filters.

In the selected 10-bed/28-object, 38-input profile, the direct LFE is removed, leaving 9 bed and 28 object tails. The frozen typed route makes bed diffuse zero; only object tails can satisfy the selected active condition. Thus the schema has 37 potential slots and the selected object branch has at most 28 active instances, not a fixed 11 or a guarantee that all 28 are active. An object tail is active only when diffuse is nonzero, width is positive, and quiet is zero. Inactive instances and their four saved outputs are removed; later reactivation starts a fresh cold instance. A first active call consumes previous raw PCM followed by current raw PCM; a warm call consumes current PCM and uses the saved prior four current-output halves. Cached drains repeat the cached input/control but advance the same lifecycle. Complete state is staged under the F03W/spatial-control render owner and committed by its parent; component/receiver state remains separately owned.

The 48-row previous-current audio history includes the 11 prefix rows; that audio history is distinct from diffusion filter state. Previous prefix contributions are recomputed under F04A.4 rather than turning each prefix into a separate filter owner. A mode/lane selector or eleven gain values alone does not supply the filter banks, ring, indices, envelope, gain, fade state or saved outputs.

## Bed control words

F01E.5 gives DBMD per-physical-track `enum48`, `enum52`, and `flag64`; F01E.6 binds bed identity to physical channel via CHNA/channel-format identity. F01E.9 defines a complete 13-word control52 for the selected **object** event branch. It does not give a full 13-word physical bed row. The selected-constructor provider's phrase `control38[38][13]` is a shape declaration; it does not by itself define the final seven bed words.

The rehashed frozen regular-control model constructs the bed prefix as six U32LE words `[0, 0, 3, 2, 1, bed_id]` for the selected typed route. Word 0 is mute, word 4 selects the bed branch, and word 5 is the bed ID; the model treats the remaining 28 bytes as undefined and unconsumed. It sets the derived bed width to +0; the prefix schedule treats bed diffuse as zero. This is a model-defined typed six-word contract, not proof that the native memory's trailing bytes are zero, absent, or have any particular value. Those seven words are `NOT_RECORDED` for this physical bed-control branch and must not be serialized as a recovered full control52 or read by a consumer without a separate source rule.

The separate F01 selected geometry source/panner record has its own defined fields and coordinates. Its fields cannot be used as evidence for the undefined tail of the physical bed control row merely because both views are associated with the same bed. The F01E object control52 has its own complete word meanings and must not be applied to bed by shape. DBMD fields remain bound by physical host track and must not be inserted into object slots 11/12 or invented as bed-tail values.

The approved constructor's callback-local control38 shape and any consumer demanding all 13 words for bed therefore need an explicit typed representation/defined-field mask or a separately justified local fill rule. A local fill would be an implementation decision, not a recovered native value. Whether any production consumer actually requires bed words 6..12 remains UNBOUND here; do not globally assert that none exists.

## Scope

The fixed 38-channel selected profile and the cited frozen route are the scope. Other layouts, dynamic ADM events, native byte contents beyond the six model-defined bed words, native owner identity, real ADM execution and listening remain outside this evidence. This is not a claim that a 36-channel sample is invalid, nor that an output 5.1 layout follows from physical input track count.
