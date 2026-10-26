# Installed target → fit/context → outer → compact matrix

Result: existing approved contracts cover this interface. No additional producer value, numerical formula, native identity, receipt type or gate was found necessary. This is a path-free field crosswalk, not a numerical rerun or implementation review.

## Approved authorities

The approved main manifest identity is `c7cddb7d5ac073c0f410fe44be9a431cfbb21c3df469d8b43c32a4297298674a`. All eight referenced sources were independently rehashed against it.

| ID | Document | SHA-256 |
|---|---|---|
| F03G | Source gate and target installation | aeb6791967413e71518cfb1c5853c52cd8e6e7e7174f60afa611cc0e0fa8d9dc |
| F03U | Target refinement and projection | ca235e2f373d95d5c628ca61c651d01805bb9cb14a7c3457d9a77b1d87941879 |
| F03P | Point response and target context | 8fb727170fb41df5354a17429fc00411010f010cb32dc86e0fdcc81bddaf4964 |
| F03N | Rectangular NNLS initial fit | 7cfb51e8905f739e60bebcd30899887bda52cc493a5bab0c976ca645548b09fc |
| F03S | Outer energy correction | 7ca47864c9fe7573b6cc6edfafdbd2049c5e74aa73f9b240c4c15bf4a427cdb2 |
| F05H | Fit control, marker and metadata history | 39cb2ef34ef976b369c4390969d6a38e291d3268e9e2c7e43499ea299b692f26 |
| OWNER | Selected owner projection and J05 | 9ff01aca7b6be6e89511e4d493f9fc9e661a50e2f098065f4a32c3fed777542f |
| GTC | Global transaction contract | 5415b7b2fb60f50351cd26691e7ef0fc5bd758e501d305d0a714b90c714cd105 |

## Field crosswalk

| Consumer input | Producer and exact binding | Existing clause |
|---|---|---|
| Installation selected IDs, attributes, XYZ | Selector order; optional U04 supplies projected F32 XYZ with the original selected ID/attribute association. Preserve protected-anchor XYZ when attribute bit0 is set. No pre-install Q15 conversion. U04 is called only for `CAPACITY_REACHED`; count-changing dedup is rejected. | F03U.1, .2, .5; F03G.2 |
| Installed target snapshot | Installation returns all15 current/previous XYZ, IDs, attributes and assignment-active flags. Mapping is old slot→selected ordinal, not a compact solver ordinal. Matched slots copy old current to previous before installing; unmatched semantics follow the n>0/n=0 distinction below. | F03G.2–.3 |
| All15 initial fit records | For every installed slot, F05H.1 packs current target XYZ and packed attribute A into14 U32 words. Words0/1 start0; words2/3/4=(A>>4)&15,(A>>8)&3,(A>>10)&3; word5=0; words6/7/8 copy XYZ F32 bits; word9=A&1; words10/11=0; word12=(A>>1)&7; word13=1. Fit records are not Type11 reader records. | F05H.1 |
| Point/context fit fields | Consume words6/7/8 as XYZ,9 as mode,10 as F32 width bits,11 as extra,12 as spatial_enum,13 as coordinate_parameter. Require mode0,width0,extra0,enum0,parameter1 for every slot, including inactive slots. Words0–5 are not point/context numerical inputs. Do not numerically convert an arbitrary U32 width to float. | F03P.6 |
| Full context | Compute one15-observation point response for each of all15 fit records, then threshold per F03P. This gives full_context[basis15][slot15] before marker modification. Use existing frozen P15/angle route; do not substitute source-response columns or P11 prefix gains. | F03P.2–.4, .6 |
| Destinations and n | Gather installed assignment-active slot indices in increasing slot order. `n=len(destinations)`. Each solver ordinal k maps to physical target slot destinations[k]. n is not this callback's selected_count and not fixed capacity15. | F03G.3; F03P.6 |
| Compact context and target geometry | A[basis][k]=full_context[basis][destinations[k]], shape15×n. target_XYZ[k] comes from the same installed current target snapshot at that slot, before marker/history updates. Keep the original finite F32 XYZ; do not substitute panner-quantized coordinates, delivered metadata or next-generation marker geometry. | F03P.6; F03S.1 |
| Outer call_order and response b | call_order is exactly the increasing processing/solver-call global-source sequence from gate. uncalled_sources is its complement in0..47. For each source s, b=already generated response_columns[s],15 F32 values in stable P15 basis order. Candidate-only sources may have response columns but are not outer solver calls. | F03G.1; F03P.1; F03S.1 |
| Outer source XYZ | Current same-callback source geometry for the global source s. This is the source installation/sampler geometry snapshot, not selector ordinal, authored audio spread, panner Q15 output or next marker XYZ. | F03P.1, .6; F03S.1, .3 |
| Per-source seed | Call F03N with the same A15×n and b15. Initialize NNLS local buffers per source; F32 inputs convert exactly to F64, arithmetic follows F03N, output seed is n F32 values. `ITERATION_LIMIT` current seed is consumed by F03S with status preserved; it is not relabeled SOLVED. | F03N.1, .7; F03S.6 |
| Per-source correction | F03S consumes A,b,seed,target_XYZ[n],source_XYZ[3]. index_state[n] starts U32 zero and scalar_flag0 per source. No previous-source/callback warm state. F03S owns energy/fallback/correction/floor/dominance behavior; numerical failure retires, never skips a source and fills zero. | F03S.1–.6 |
| Compact C and destinations | Initialize C[n][48] positive-zero, row-major by active ordinal then global source. For each processing source s write its final n coefficients C[k][s]. Uncalled source columns remain positive-zero. Output the same destinations[k] mapping used to gather A/target XYZ. No target-slot-index addressing into compact C and no capacity15 padding. | F03S.1; F03G.3 |
| Marker inputs after outer/mix | Marker consumes C with its existing active_target_indices mapping, source flags, assignment-current-active and committed previous_active, current/previous target IDs/attributes/XYZ, plus the newly constructed fit records. Quiet witness is first source with flag0 and coefficient>0. Old energy comes from committed component overlap; new energy comes from current matrix output first1536 samples. | F05H.2–.3 |
| Post-marker ownership | Quiet may clear active flags; marker changes fit words0/1 and supplies signed marker. It does not retroactively reorder solver ordinals or re-gather the already computed C rows. Metadata history mutation belongs uniquely to F05R56 under OWNER, not to the context/NNLS/outer producer. | F05H.2–.4; OWNER Callback split2,5–7 |

## Required snapshot order

1. Hold the current callback's source geometry and gate response proposal.
2. Use selected or permitted U04 projected XYZ/IDs/attributes to prepare installation from committed target state.
3. From installation's current15-slot snapshot, construct and validate all15 fit records and full15×15 context before F05H marker changes.
4. Gather installed active slots once to destinations, compact A and compact target XYZ. For each processing global source run fresh NNLS then outer, filling C[n,48].
5. Apply the existing matrix adapter/mixer with C and these same destinations. Obtain the current first1536-sample target audio for marker energy without committing overlap twice.
6. Generate quiet/marker using pre-marker installation and committed previous-active/overlap inputs. Prepare metadata history through its unique owner. Parent transaction commits staged owners once only after all required downstream steps succeed.

Context calculation is pure. It does not advance target history, feature history, metadata slots or any extra callback clock. The local transaction/receipt connection is an existing LOCAL_MODEL interface, not a request to recover native DEE record identities.

## Empty and retained-state cases

- Empty selection: F03G.3 returns all mapping entries−1 and preserves current/previous XYZ, IDs, attributes and active flags in full. Old active slots may therefore give n>0. Never derive n from selected_count or clear active because selection is empty.
- Nonempty selection: unmatched slots become assignment-active0 but preserve their current/previous geometry and identity fields; matched slots install selected values. This differs from the empty-selection rule.
- n=0: still construct and validate all15 fit records/full context under F03P.6. Then project A to15×0, target XYZ/destinations empty, return C0×48 and make no NNLS/correction calls (F03N.1, F03S.1). Inactive-slot invalid context branches are not waived.
- n>0 but no processing sources: validate context/geometry and return all-positive-zero C[n,48], no per-source NNLS/outer call. Marker/quiet, target retention and downstream callback ownership still proceed; this is not whole-callback early return.
- Empty candidate set does not imply empty processing sequence or empty retained targets. Gate, selected, processing, installed-active and final quiet-active are distinct domains.

## Failure and verification scope

All required consumption values and equations are already specified. No frozen helper or DLL was needed. This audit rehashed the eight approved sources and checked field/shape/order references; response, NNLS/outer and matrix numerical tests were not repeated. Isolated implementation source, real audio, native execution and production interface execution were not read or run.

GTC and OWNER are authoritative over older local F05H.6 retry wording: failed production callback/source discards staged roots, retires the attempt and permits only complete programme-origin restart. Do not add same-index production retry, independent leaf commits or new native identity gates.
