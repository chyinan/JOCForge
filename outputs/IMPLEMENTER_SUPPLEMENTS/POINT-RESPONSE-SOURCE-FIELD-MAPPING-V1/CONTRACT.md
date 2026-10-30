# PointResponseSourceInput: four U32 source fields

Existing F01GL.6 plus the hash-matched frozen source writer and point reader close this mapping. Relative offsets below are schema-field positions, not executable addresses. They do not require recreating an opaque native source record or supplying its undefined leading hold field.

| Semantic field | Point reader relative source byte offset | Object13 Control52 slot | Prefix/bed selected producer | Object selected producer |
|---|---:|---:|---|---|
| feature_mode | 36 | 8 | Explicit U32 zero at geometry/source initialization | Copy same-callback Control52 word8; F01E.9 currently supplies0 |
| source_skip | 4 | 0 | Explicit U32 zero in the defined caller/source initialization | Copy same-callback Control52 word0; F01E.9 currently supplies0 (enabled) |
| spatial_enum | 48 | 11 | Explicit U32 zero at geometry/source initialization | Copy same-callback Control52 word11; F01E.9 currently supplies0, the selected component branch |
| coordinate_parameter | 52 | 12 | Explicit U32 one at geometry/source initialization | Copy same-callback Control52 word12; F01E.9 currently supplies1, Z enabled |

The object writer covers source bytes4..55 with the complete52-byte control, so slot j has relative offset4+4*j. The actual point reader independently selects offsets4,36,48,52. This is a writer-to-reader relation, not inference from equal selected constants. coordinate_parameter remains a copied input; do not implement it as unconditional1. The point primitive admits0/1; selected ordinary Object13 currently constructs1. A transport discriminator with slot12=0 must read0, while this does not add a new admitted normal-control producer.

Adjacent fields clarify the interface:

- XYZ is source bytes24..35, exactly Control52 F32 slots5..7. Preserve their current sampler F32 words. Bed/prefix use their explicit installed geometry instead of an object-control overlay.
- Authored audio width is source bytes40..43, Object13 F32 slot9. It is not feature_mode or any U32 enum. Preserve this width for audio routing/F04G.
- Object13 slot10 is diffuse, relative source44. It is not spatial_enum, coordinate_parameter or feature_mode. DBMD enum48/enum52/flag64 are not copied into these slots: F01E.5 and .9 explicitly keep them separate.
- Source relative8 is Object13 slot1, an unused own zero in the selected interface. Despite being zero in ordinary records, it is not the point reader's mode field at relative36.
- Source relative0 is the leading hold/state field. It is neither source ID nor source_skip and is not read by these point status/numeric helpers. It remains undefined by this scoped model; do not reconstruct a complete56-byte native object or invent its warm transitions.

## Writer order and feature view

1. Install frozen prefix geometry and normalized bed geometry by semantic identity. Prefix owns global0..10. Sort physical hosts0..37, skip the unique semantic bed LFE, compact the remaining37 to global11..47. Bed/object roles follow channel identity, not assumed contiguous host ranges. No LFE source enters this regular domain.
2. At each normal callback, F01E sampler yields the complete object Control52 snapshot. Object installation copies that same snapshot to the defined source tail. The four U32 fields and XYZ come from one callback/physical-source association; no join by equal values or unrelated Object ordinal.
3. Audio routing consumes the authored width. At the feature consumer boundary create a separate view with feature_width positive-zero at relative40. The frozen post-view helper reads and requires pre-route mode at relative36 to be0, then writes width only. Under the selected branch, mode clearing/preservation paths all have the already-proven value0. Do not use this as permission to convert arbitrary nonzero mode into0.
4. Point input reads mode/skip/enum/parameter from this view at the table offsets, with width from its positive-zero field. The width transformation does not overwrite source_skip, spatial_enum or coordinate_parameter. The model has no additional selected point-view warm transition of these three fields. It does not establish universal native hidden-state behavior.
5. Check all48 selected feature modes0. For actually generated union sources require source_skip0, feature_width0, spatial_enum0 and coordinate_parameter0/1, as in F03P.1. Source_skip is the raw source-control field, not processing_inactive/selection_suppressed flags from gate.

## Callback and cached-drain provenance

F01E.10 owns sampler/event state and the complete last-normal Control38/regular37 snapshot. F03W consumes that same callback snapshot. Main cached drains reuse the final normal controls and source association, without re-sampling ADM events. The drain callback generation advances, while its control data provenance remains that last-normal snapshot. Recreate the point feature view per callback; never mutate the preserved audio/control snapshot when clearing feature width.

Prefix/bed geometry is programme-installed and frozen; object controls update on normal callbacks and are held on main drains. Successful parent callback commits once under existing ownership. Errors expose no partial point proposal and follow I03. No new receipt/digest or extra source-state owner is required. Independent meter uses its own drain-control contract, outside this main held-control statement.

## Exact sources and input checks

- F01GL.6 specifies prefix/bed defined fields, object offsets4..55 overlay, global48 identity routing and the unclaimed leading hold field.
- F01E.9 gives typed13-slot production, including slots0/8/11/12 and distinct F32 width9; .5 excludes DBMD aliases; .10 gives current/held/reset snapshot ownership.
- F03P.1 gives selected feature-view transformation and consumption checks; .7 makes columns callback-local rather than state history.
- Frozen writer/reader/post-view identities and source line ranges are recorded in SOURCE-IDENTITIES. No frozen code, executable coordinates, PCM or response bodies are delivered.

Input-only probes independently tag slots0/8/11/12, retain a parameter0 transport case, and give nonzero audio-width9, unused-slot1 and diffuse10 to distinguish false aliases. These mutated transport probes may be outside admitted selected control domain; they are not alternative production defaults. Synthetic installation covers every one of38 LFE physical positions with interleaved bed/object roles and exact global48 association.

Normal and optimized verification each compare7296 U32 fields over1824 generated source records, plus6 transport discriminators. The undefined leading hold field remains unread/unfabricated. This proves the scoped frozen-model installation-to-reader mapping, not native runtime or real ADM execution.
