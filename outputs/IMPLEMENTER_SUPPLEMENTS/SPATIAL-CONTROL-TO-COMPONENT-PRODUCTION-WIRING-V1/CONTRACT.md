# Spatial control to component production wiring

Ready scope: connecting existing selected F03/F04 numerical providers into the current-callback installed48/matrix/component producer and target-control handoff. This is a model implementation contract, not a claim that a full DSP callback or native programme has just executed.

## Inputs and unique owners

Consume physical38 and the admitted current F01E control/sampler snapshot through the approved UID-to-physical-slot map. Physical38, regular37, prefix11, installed48, candidate ordinals, global source IDs, active matrix ordinals and fifteen target slots are distinct domains. Bed/object physical slots may interleave. Do not hard-code objects at physical slots10..37.

The selected ADM event scope remains F01E's single zero-start programme-covering event per object. Use its unique sampler state/provider and current outputs. The frozen worker's pre-loop first-record fixture is not a general event scheduler, and constant coordinates alone do not authorize skipping a blend. Only an actually established bitwise fixed-point predicate can justify an equivalent optimization; finite example replays are not a universal theorem. Meter zero-input drains follow the existing F06 parent input/control contract and do not create a new event-time advance or import the main cached-drain policy.

Reuse the approved owner projection. Frontend control/sampling is not duplicated inside spatial control. The spatial-control stages own their declared render/flag/feature/target/matrix state; the F05R56 stage is the sole mutator of its marker/history/reader state under its parent admission. Component/filter/overlap state has its existing spatial-audio owner (or independent meter counterpart). Do not copy the research front's bundled history or diagnostic LFE state into a second mutable owner. Component.before is immutable during marker preview.

## Current-callback pipeline

1. Render installed48[48][3072] and record the actual executed prefix writer set. Produce SourceFlags from the same physical input, controls, old flag history and current prefix values. Produce raw band-major features[24][48] from that installed48, flags and feature.before.
2. Sum each source's 24 F32 bands serially with a separate R32 addition at each step. Candidate IDs require plus4==0 and sum>=R32(0.5). Processing IDs independently require plus0==0. Both lists are source-ascending.
3. Generate response columns for the sorted unique union of candidate and processing IDs. Each consumed column has fifteen observations. Candidate widths come from the post-audio feature-source view. Do not fabricate missing consumed columns or read undefined fields that this path does not consume.
4. Select from the original candidate feature snapshot, geometry/attributes, responses and previous installed IDs. Keep global IDs and selected order. Run accepted U04 only for CAPACITY_REACHED. CANDIDATES_EXHAUSTED skips it even when selected_count=15; NO_CANDIDATES also skips. U04 uses the pre-decay features and must preserve selected count, IDs/order and attributes.
5. Install/refine the selected targets using the approved cost/greedy rules. T=0 preserves old current, previous and active arrays. T>0 retains the existing attribute multiplicity and finite-cost admission; do not invent a fallback cost or special table. Capture ascending pre-quiet active slots d and n=len(d), independently of T.
6. Construct and validate all fifteen fit records and full context[15 basis][15 slots], including inactive slots. Gather context[15][n] and target XYZ[n][3] by d. For each processing source use its full response, fresh zero correction-index state[n], and scalar_flag=0 in the approved selected-point branch. That scalar flag is not either SourceFlags field.
7. Generate a fresh compact C[n][48] with positive-zero uncalled columns through the existing outer/NNLS/correction rules. n=0 is a produced empty result and invokes no solver. n>0 with no processing sources is a produced all-zero result. Neither case permits reading a cold placeholder or using nonzero coefficients as readiness.
8. Use current installed48 and C/d for a pure marker mixer preview and read component.before overlap for marker energy. This preview advances no component/filter state. Quiet-after affects next target state, not current C/d. Submit marker/history inputs once to the unique owner. Publish canonical D and the true active d under the already approved representation adapter.
9. Execute the existing component stage once on the same installed48/C/d and its before states: 24 blocks of64 samples, fifteen output rows. Empty or zero C does not skip historical overlap/filter/parameter updates. Return the real component15 and same-callback controls to the existing independent meter LFE/OAR bridge.

The attached stage table gives exact input/output axes. Generated proposals become available only when all their required producers for g completed and current values, provider identities and extents validate. Copying a zero cold matrix does not complete a stage. Do not rerun a producer merely to regenerate its receipt.

## Target controls, history and row identity

For target slot s=0..14, build the fourteen-word fit record by the approved packed-attribute/XYZ rule. Marker writes quiet at word0 and inactive_pair at word1. At every callback, history begins nextS0=oldS1, nextS1=oldS1, nextS2=oldS2; it is not a full FIFO shift. Marker -1 inserts new into all three slots, 0 into S1/S2, 1 into S2, other I32 values insert nowhere. After insertion, if S0.word1 and S1.word1 are nonzero, copy the entire S0 record and its lineage to S1.

Deliver updated S0 and project exactly bytes[4:56]: control52[s].word[j]=fit.word[j+1]. Quiet is omitted; control word0 is inactive_pair, not a wire mute flag. All fifteen history rows retain their target slots even when the active matrix has no rows. The 37 physical-source controls upstream are a different type/domain and cannot substitute for these fifteen target controls.

Fit-control record56 and decoder reader record56 also have different typed layouts. Their equal byte length does not authorize reinterpretation; use the approved Type11 encode/decode and reader conversion path where required.

The meter control16 row1+s is that exact thirteen-word delivered target control. Direct row0 is the canonical owned [0,0,3,0,...,0], with only word2 consumed. Target updates row1+s retains the same target slot; compact matrix row k identifies the slot only through d[k]. Do not filter/reorder these control rows by quiet-after or a mixer padding permutation.

History/target snapshots advance at g=0,1 while reader invocation and target_updates receipt are absent. At g>=2 reader q=g-2 executes. A delivered regular control retains S0's original producer generation/row, potentially older or ColdConstructor. ColdConstructor is not callback0; encode it according to the named boundary (-1 or null), without treating legacy diagnostic strings as product IDs. Packet generation is g+1; direct metadata provenance is current g; LFE audio provenance after priming is g-2. These values must not overwrite one another.

## Commit and admission scope

Use the existing clean staged-root/parent contract. F05R56 emits its next-state proposal and immutable callback-local receipt; it cannot admit or commit itself. The spatial_control parent commits its owned next state and admits that receipt at its existing commit point. Consumers requiring admission cannot use a pending receipt. Larger programme publication remains governed by the existing transaction hierarchy; this supplement creates no new transaction framework or early global publication.

No partial externally committed state or success receipt survives a failed staged callback; preserve the approved retire/full-reset behavior. A legacy production-admission=false report remains historical execution status, not proof that a DEE serializer or identically named receipt must be found before implementing these clean model stages. It is not changed into an execution PASS here. Actual production tests still decide successful integration and encode readiness.

The verification exercises source identities, metadata gates, empty-target guards, index/bitfield mapping and synthetic history inputs. It does not execute nonempty NNLS, full response/selector/U04 numerics, marker energy, render/component DSP or a real ADM callback. Those algorithms remain the existing approved numerical providers, not placeholders supplied by this wiring package.
