# Object track identity to local sampler owner slots

This is a small internal representation binding for the existing F01E normal producer. It supplies the missing association from admitted graph track identity to the28 nested sampler-owner positions. It adds no DSP rule, native state-array claim, new digest or broader ADM admission restriction.

## Evidence domains

F01G defines semantic_key once: parse the admitted AO_4hex value and subtract0x1001. A Prepared semantic_key is already that result. It is not an ordinal, physical host or a value to subtract again.

The rehashed event producer iterates Objects track rows from the graph, retains that key alongside TrackUID/host identity and constructs raw96 row_index from physical host. It does not use semantic_key to index a28-entry sampler array.

A different rehashed model function filters Objects tracks, sorts by ASCII audio_track_uid bytes and enumerates them to produce the meter source descriptor's group_word_offset4. Its actual caller is separately pinned. It then sorts descriptor output by physical row. This establishes UID-sort group ordering in that descriptor domain only. It does not identify the clean F01E sampler's array order or make all same-valued ordinals interchangeable.

The current F01E constructor's object_ordinal0..27 is a local owner slot. Its existing rules require a unique association to runtime physical binding but had not prescribed which graph record gets each local slot. The rule below is an explicit clean internal allocation decision.

## Minimal deterministic local binding

At programme preparation, take exactly the28 admitted Objects TRACK records. Do not reduce them to distinct semantic keys or AO IDs: F01G can associate one AO with multiple TrackUIDs, and each admitted object track still has its own physical control/sampler owner.

Using only existing admitted TrackUID type and uniqueness rules, sort by unsigned ASCII bytes, with normal lexicographic prefix/length behavior. Enumerate once into local_owner_ordinal0..27. Store or retain the complete immutable association:
TrackUID ↔ local_owner_ordinal, plus that track's semantic_key, host_channel_index and channel_format_id.

The table belongs to existing immutable Prepared graph/physical-binding data. Mutable sampler state stays the existing28 nested entries. Do not add another sampler history or a new digest solely for this map. Freeze the mapping for the programme/attempt; reuse of old state under a different mapping must follow the existing reset/failure policy.

This local order is chosen for determinism and convenient explicit joins. It is not a DEE sampler-order assertion. If a consumer already has a consistent local bijection, use its explicit UID→slot map or convert references by UID; do not force reorder its values or reinterpret an existing state array from coincident numbers.

## Runtime and consumers

Normal processing still visits objects in ascending physical-host order. For each track, look up its TrackUID in the frozen map, then step that specific nested sampler state and produce the physical control row. State-slot order and invocation order are separate.

Control38, regular37, raw96 physical/type rows, spatial global source IDs and selected target IDs keep their previously defined domains. F01E control52 contains sampled/control values, not this clean owner index. Semantic keys stay unchanged for their own event/metadata consumers.

The meter descriptor's externally defined group order is still computed by its own UID-sort rule. If an adapter needs descriptor group→sampler owner, construct an explicit join using exact TrackUID and its verified host/channel association. Under the chosen local order the numbers happen to match; equality is not the proof of identity.

Graph/document/master-child order does not define the owner. Neither does sorting semantic keys or physical hosts. Noncontinuous keys are valid when admitted by existing F01G. No new TrackUID spelling rule, contiguous-key requirement or semantic-key uniqueness check is introduced here.

## Small verification scope

Six complete self-owned28-track inputs cover noncontinuous keys, reversed/rotated graph order, reversed hosts, a shared AO/semantic key with two distinct TrackUIDs, and the existing general ASCII UID type. The verifier executes only the real frozen descriptor function's filtering/sorting/group-enumeration prefix, plus the explicit clean local map. It checks preserved per-track keys/hosts/channel associations, independent host-order traversal and conversion from a pre-existing alternative bijection.

The source prefix test is not a full descriptor/native execution. No actual ADM/PCM, sampler numerics, native state array, implementation code or backend work is read or executed. The normal snapshot and parent-commit contracts remain unchanged.
