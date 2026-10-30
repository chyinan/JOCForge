# U04 membership gain producer: actual caller join

The production rule already exists: F03U.3 produces the per-source geometric gain row and F03U.4 names that result membership_gain[N][T]. A project_targets subinterface taking gains as input does not make the complete F03U producer caller-supplied or undefined. Use those approved equations, not a synthetic projection fixture as production data.

## Inputs at the capacity exit

| Field | Actual source and identity connection |
|---|---|
| N and candidate IDs | Original increasing global-source candidate sequence from gate at this callback, before selector survivor compaction. Not solver processing order, response union, selected count or final survivor list. |
| source_xyz[N][3] | Same-callback geometry gathered by those original candidate global IDs; preserve F32 values and convert exactly to F64 for F03U.3. No panner Q15, authored width or old target geometry substitution. |
| source_attributes[N] | Complete U32 attributes gathered by the same candidate IDs and ordinals. The gain producer consumes the full source attribute, even though project_targets itself does not take this array. |
| original feature24×N | Frozen candidate feature view before selector decay. Historical Python caller explicitly retains gate feature values while selector has separate working data. R1 owns the numerical source_weight provider. |
| T and selected IDs | Current capacity path T=15; selected global IDs in selector selection order. Retain their association outside local debug transport labels; do not sort them or interpret local target ordinal as global source ID. |
| initial target_xyz[T][3] | Gather the original same-callback geometry using each selected global ID, before U04 projection and before target installation/assignment. One fixed target anchor snapshot for the whole source loop. |
| target_attributes[T] | Complete U32 attribute for each selected global ID in that same selected order. Equality/penalty in F03U.3 uses the full attribute; bit0 has its separate later protected-anchor projection role. |

The frozen Python caller and fixture adapter independently establish these identity gathers and shapes. The older artifact projection-control utility reads gain from a saved solver output; that utility is a consumer, not proof of a production gain formula. Current branch admission is defined by approved F03U.1 and corroborated by the cached native selector exit, not by a historical fixture wrapper's broad execution label.

## Actual call and loop evidence

The cached native selector builds its own working feature/geometry/attribute/survivor buffers, while retaining original input descriptors. Its exhausted-survivor exits return before refinement. Completing the capacity loop passes the original source descriptor/count, original attributes and completed selected-target descriptor/attributes to the refinement wrapper.

The wrapper allocates local target XYZ and membership scratch. It copies the selected target points and full attributes before the source loop. For original source ordinal j=0..N-1, it reads source_xyz[j] and source_attributes[j], invokes the geometric gain producer against that fixed target snapshot, obtains a T-value F32 gain row, then performs the existing projection weighting. Candidate global identity is carried by ordinal j; target identity is selected ordinal i.

The geometry gain helper reads source/target F32 values into its own F64 working arrays, with target attributes copied along the same target-index selection. Its output scatter returns each F32 gain to its target ordinal. On the approved unchanged-dedup-domain path, these ordinals map one-to-one to the original selected targets. Existing F03U.2 rejects count-changing dedup in the clean caller; do not infer a broader remapping API from native local scratch.

All source rows complete before final projection matrix multiplication writes projected targets. No per-source projected target is fed back as the next source's anchor. Initial anchors, full target attributes and N-order stay fixed. It is one U04 solve/projection pass per admitted callback, not an iterative target update or history/warm-start procedure.

## Formula, precision and handoff

Apply F03U.3 verbatim per source: build its F64 geometry system with full-attribute penalty, independent F64 operations in the specified axis/loop order, lower Cholesky and forward/back solve, then gain_i=R32(max(-solution_i,0)). No NNLS/context/response or F03S correction substitution. Reuse approved F03U.2 dedup and failure boundaries; singular/nonfinite/count change must fail-stop, never fabricate zero rows.

Materialize G[j][i] in source-major N×T order from these raw gain rows if the existing project_targets interface requires the full array. Do not pass the later normalized/squared projection matrix M[T][N], output target XYZ, F03S coefficients or cached solver artifacts as membership gain. R1's source_weight[N] must have the same original candidate ordinal j. Remaining normalization/projection/protected-anchor behavior stays F03U.4–.5 without changes.

Call only when selector termination is CAPACITY_REACHED. CANDIDATES_EXHAUSTED skips refinement even when selected_count happens to be15. No candidates likewise skip. U04 inputs and scratch are callback-local; projected XYZ retains original selected ID/attribute order and passes to installation. The existing parent callback owns the single commit. No native receipt/hash identity or additional state owner is required.

## Evidence and bounds

SOURCE-IDENTITIES binds approved F03U/F03C/F03G/N00, the frozen Python caller/adapter, and four independently rehashed recovered native database bodies. Native evidence is CACHED_STATIC; fresh original PE rehash and runtime observation are NOT_RUN. No machine code, decompilation or executable coordinates are delivered. This audit closes caller/producer domain and timing joins, not fresh production numeric parity.

The input-only discriminator uses17 sparse candidate global IDs and15 unsorted selected IDs, distinct exact dyadic coordinates and valid U32 attribute distinctions. Structural verification checks17 source argument rows and255 source-target joins, full attribute equality, fixed initial anchors and selected-order preservation. It supplies no expected gains. Source-weight mathematics is intentionally left to R1; the approved gain formula was not rederived or replaced.
