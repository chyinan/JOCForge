# F04R receiver pipeline and final-six map

由 IMPLEMENTATION-MAP.json 生成，不得手工修改。

Mapping status: `MAPPING_COMPLETE`. Implementation readiness: `LOCALLY_VERIFIED_F04R_DIRECT_RECEIVER_ONLY`. Audit status: `LOCALLY_VERIFIED_F04R_DIRECT_RECEIVER_ONLY__WHOLE_RECEIVER_NOT_RUN`.

This is a mapping-only gate. It does not implement Rust/DSP, JOC/J05, F05JC, codec behavior, a parent transaction, or any root commit. The F04R route is executed once by F04C for each callback whose Type10 outcome is Ready.

## Callback and lanes

The receiver consumes `16 rows x 1536 F32 samples.` from the E4 packet/receipt pair for `g>=2`. row0 is the delayed direct-LFE source for source s=g-2; rows1..15 are component15[g][0..14] for the current callback g. Required contract input is same-callback target_updates[g] from the F05R56 target-owner receipt; same-runner producer wiring is NOT_RUN. Update only each supplied row.target[6], retaining current/slope/remaining; rows without updates retain their old target.

| output lane | identity | route |
|---|---|---|
| 0 | L | Output lanes0/1 are L/R; they participate in QMF analysis history only, not mode=1 delay/FIR/synthesis. |
| 1 | R | Output lanes0/1 are L/R; they participate in QMF analysis history only, not mode=1 delay/FIR/synthesis. |
| 2 | C | Output lane2 is C; it participates in QMF analysis history only, not mode=1 delay/FIR/synthesis. |
| 3 | LFE | Output lane3 is LFE; it receives the dedicated four-section post-filter and bypasses QMF analysis and synthesis. |
| 4 | Ls | Output lanes4/5 are Ls/Rs; these alone receive receiver mode=1 delay, band0 FIR and QMF synthesis. |
| 5 | Rs | Output lanes4/5 are Ls/Rs; these alone receive receiver mode=1 delay, band0 FIR and QMF synthesis. |

## Ordered callback pipeline

| order | stage | authority | input -> output | state fields |
|---|---|---|---|---|
| 1 | F04R.3 scalar preconditioning | F04R producer, called once by F04C for this successful receiver callback | Type10[16][1536] -> preconditioned[16][1536]. | ['ModelState.filter_state'] |
| 2 | F04R.4 target installation and row/lane accumulation | F04R producer | preconditioned rows plus target_updates[g] -> accumulator[6][1536]. | ['ModelState.row_states'] |
| 3 | F04R.5 transform delay, PG01 blend, first ring | F04R producer | accumulator[6][1536] -> six-lane pre-LFE-filter candidate[6][1536]. | ['ModelState.transform_delay', 'ModelState.first_ring'] |
| 4 | F04R.6 LFE post-filter | F04R producer | pre-LFE-filter candidate lane3 -> filtered lane3; lanes0/1/2/4/5 pass unchanged; combine as s09_input[6][1536]. | ['ModelState.post_filter_state'] |
| 5 | F04Q receiver analysis and mode=1 output | F04R producer through the separate F04Q stage | s09_input[6][1536] with lane3 already post-filtered -> QMF analysis histories and updated Ls/Rs PCM scratch. | ['ModelState.analysis_history', 'ModelState.qmf_delay_history', 'ModelState.qmf_fir_history', 'ModelState.synthesis_history'] |
| 6 | F04R.7 temporal0 then temporal1 then temporal2 | F04R producer | QMF/post-filter scratch -> post-temporal0..2 scratch for descriptor selection. | ['ModelState.temporal_states[0..3]'] |
| 7 | F04D.3 descriptor selection before temporal3 | F04D substage, executed within the F04C-owned callback | post-temporal0..2 scratch -> selected index/control proposal carried across temporal3. | ['ModelState.descriptor_state'] |
| 8 | F04R.7 temporal3 | F04R producer | post-temporal0..2 six-lane scratch -> post-temporal3 scratch. | ['ModelState.temporal_states[3]'] |
| 9 | F04D.4 descriptor gain ramp | F04D substage, executed within the F04C-owned callback | post-temporal3 scratch -> descriptor-gained six-lane output. | ['ModelState.descriptor_state'] |
| 10 | F04R.8 post-S09 delay and half scale | F04R producer | descriptor-gained six-lane output -> delayed and half-scaled six-lane output. | ['ModelState.post_s09_delay'] |
| 11 | F04R.8 second ring and final_six | F04R producer; output remains subordinate to the enclosing F04C callback | delayed half-scaled lanes -> final_six[6][1536]. | ['ModelState.second_ring'] |

## Delay, temporal, and ring transitions

| transition | exact rule |
|---|---|
| transform_delay_and_blend | For each lane and 32-sample block, emit the old 32-sample delay and replace it with the current accumulator block. With W=receiver_blend32 and a5=a6=2^-8, evaluate left=(x[j]*a5)*W[31-j], right=(x[j]*a6)*W[j], output=(left+right)*256 using the specified separate F32 operations; do not merge complementary-window products. |
| first_ring | Clamp each finite input sample to [-1,1], write at write_index, read at read_index, then advance both independent cursors by one modulo 225. Cold write=224/read=0 yields a 224-sample actual delay; this is a distinct ring owner. |
| temporal_exchange | index_28 is the semantic read cursor; index_32 is the semantic write cursor; stride_bytes is 4. Both cursors use the inclusive range 0..2*span; 2*span is valid and is not eagerly reduced. For N=1536, count=min(span,N), write_first=min(count,2*span-write), read_first=min(count,2*span-read); write the last count input samples with one wrap before reading count outputs from the updated buffer with one wrap, then append input[0:N-count]. next_write=write+count if count==write_first else count-write_first; next_read=read+count if count==read_first else count-read_first. |
| post_s09_delay_and_half | After the descriptor ramp, per lane output old_delay[32] followed by current[0..1504]; set next_delay=current[1504..1536], then multiply the complete 1536-sample lane by 0.5. |
| second_ring | Apply a second independent ring after half scale, with the same clamp/write-before-read/cursor-modulo-225 rule as the first ring; cold write=224/read=0. Do not alias the two ring states. |

## Persistent ModelState owner

Owner: `spatial_audio.F04CompositionState.receiver_state: ModelState`. Canonical schema: `test-vectors/F04-WHOLE-SPATIAL-STATE-SCHEMA.json` (226344 bytes, SHA-256 `4fdab9f591821d7c50e7daca17bc98d4a5b0616e9cb45b5937cae47b26ed657c`), pointer `#/whole_state_by_boundary/cold/fields/6/schema`.

| field | canonical schema / dimension summary | cold/reset |
|---|---|---|
| filter_state | 16 F32 positive-zero scalars | all positive zero |
| row_states | 16 RowState values; each current/target/slope is 6 F32 and remaining is 6 U32 | all F32 positive zero; remaining all U32 zero |
| transform_delay | 6 lanes x 32 F32 | all positive zero |
| first_ring | 6 RingState values; words=225 F32, write_index/read_index are I64LE | words positive zero; write=224, read=0 |
| post_filter_state | 4 sections x 2 F32 | all positive zero |
| analysis_history | 5 rows [L,R,C,Ls,Rs] x 9 x 64 F32 | all positive zero |
| qmf_delay_history | 2 rows [Ls,Rs] x 10 x 128 F32 | all positive zero |
| qmf_fir_history | 2 rows [Ls,Rs] x 40 F32 | all positive zero |
| synthesis_history | 2 rows [Ls,Rs] x 576 F32 | all positive zero |
| temporal_states | TemporalState(span:I64LE,stride_bytes:I64LE=4,index_28:I64LE semantic read,index_32:I64LE semantic write,buffers); temporal0=(span1217,3 lanes), temporal1=(span63,5 lanes), temporal2=(span900,1 lane), temporal3=(span256,6 lanes), reserved temporal4=(span896,15 lanes) unchanged. | F04R.1 gives logical read=span/write=0 and two positive-zero buffers per lane; the approved F04R semantic crosswalk maps index_28=read and index_32=write; temporal4 remains unchanged in the selected route |
| descriptor_state | DescriptorState(tail_peak:F32, hold_count:I64LE, smoothed:F32, exponent:I64LE, reference:F32, gain:F32) | tail_peak=+0, hold=0, smoothed=+0, exponent=2, reference=R32(2/48), gain=1/16 |
| post_s09_delay | 6 lanes x 32 F32 | all positive zero |
| second_ring | 6 RingState values; words=225 F32, write_index/read_index are I64LE | words positive zero; write=224, read=0 |
| target_coefficients | None in the canonical schema | diagnostic/reserved sentinel only; not consumed by the selected route |

TemporalState canonical schema pointer: `#/whole_state_by_boundary/cold/fields/6/schema/fields/9/schema`.

| TemporalState field | canonical encoding |
|---|---|
| span | I64LE |
| stride_bytes | I64LE |
| index_28 | I64LE |
| index_32 | I64LE |
| buffers | lane tuple; two span-sized F32 buffers per lane |

| instance | span | lanes | buffer bytes per lane | route |
|---|---|---|---|---|
| temporal0 | 1217 | 3 | 9736 | consumed |
| temporal1 | 63 | 5 | 504 | consumed |
| temporal2 | 900 | 1 | 7200 | consumed |
| temporal3 | 256 | 6 | 2048 | consumed |
| temporal4 | 896 | 15 | 7168 | reserved and unchanged |

Semantic cursor crosswalk: `MAPPED_EXPLICITLY_BY_APPROVED_F04R_SPEC`. 

The full ModelState is staged as one after-state. F04R/F04D/F04RB have no local commit; only the enclosing F04C/spatial_audio transaction may commit it.

## Clocks and drain

| clock | contract |
|---|---|
| frontend_normal | g=0..4246 |
| frontend_cached_drain | g=4247..4250 |
| source_clock | For g>=2, source s=g-2; g=0/1 have no source-backed Type10 Ready packet for F04R. |
| source_count | 4,249 receiver sources; s=0..4248. |
| receiver_clock | For g>=2, receiver callback r=s=g-2; one successful callback advances receiver count once. |
| receiver_generation | After-state generation is r+1=g-1. |
| dispatch_clock | For g>=2, JOC dispatch d=g-1=s+1; dispatch remains outside this F04R map. |
| priming | g=0/1 advance upstream component/LFE priming only; no F04R, QMF, descriptor, or F04RB call. |
| cached_drain | All actual g=4247..4250 callbacks with Ready Type10 consume warm ModelState once; they are not synthetic flushes. |
| eof | EOF adds no new codec callback, no F04R callback, and no zero-input temporal/QMF flush. |

## F04C/F04D/F04RB and downstream boundaries

| owner or consumer | boundary |
|---|---|
| F04R | Receiver producer contract for the ordered preprocessing, mode=1 receiver, temporal, and final_six stages; state is proposed under the spatial_audio owner. |
| F04C | Unique executor of a full receiver callback and sole producer of selected-j05-owner-receipt.v1 after the whole callback; owns callback coordination and commit boundary. |
| F04D | Descriptor substage only: selects after temporal0..2/before temporal3, applies ramp after temporal3; no local commit, no J05 receipt, and no final_six input to selection. |
| F04RB | Independent mode=0 fitting branch with separate fitting_qmf_state; uses same-generation s09_input and descriptor after-index projection, never aliases receiver delay/FIR histories or consumes final_six. |
| F05JC | Downstream same-generation JOC producer consumes prepared mode=0 fitting/target/carrier inputs and projections; it must not rerun or commit F04R/F04D/F04RB state. |
| J05_F05JL | F04C assembles the selected J05 owner receipt; F05JL only consumes that projection and must not rerun F04R, F04D, or F04RB. |
| writer_and_parent | JOC writer dispatch and all parent transaction/commit actions are out of scope; no F04R-local publication or commit. |

## Provider and vector evidence

| provider | bytes | SHA-256 | bound fields |
|---|---|---|---|
| parameters/qmf-receiver-functional-v1.json | 20325 | 30e63cc110b63dd88118ad37f02750a39167bd01279a3b40853afdf3151b552c | ['preconditioning_alpha', 'preconditioning_beta', 'preconditioning_tiny_strict', 'accumulator_magnitude_threshold', 'accumulator_delta_threshold', 'accumulator_smoothing', 'lfe_post_filter_4x5', 'qwin_synthesis_640', 'band0_real_mode1_21', 'band0_imag_mode1_21', 'descriptor_threshold_127', 'descriptor_level_offset', 'descriptor_level_linear', 'descriptor_level_quadratic', 'descriptor_headroom_step', 'descriptor_result_scale', 'descriptor_rise'] |
| parameters/window-q15.json | 2704 | a12f9cbf91f1cf540c2eae77e4d236970defa70409f00ddf62ba87bfeafbb130 | ['receiver_blend32[32] as Q15 values, converted to F32 exactly for F04R.5'] |
| parameters/qmf-mathematical-fft-v1.json | 6983 | e8d3fb1514a5ffd857740222eacc8995208a29c3447e48c1b63083af368c0431 | ['QMF FFT roots and rotation used by the separate F04Q stage'] |
| parameters/descriptor-mathematical.json | 4861 | a493fbdaf2dab30e940cae9976ed3305f3af083e5614ab0cbd6889c71a02b197 | ['PG03 gain[128] for the F04D ramp; its mathematical threshold table is not the selected F04D lookup threshold'] |

| stage | direct coverage | qualification |
|---|---|---|
| F04R.3 scalar preconditioning | ['case1_cold_normal', 'case2_warm_normal', 'case3_cached_drain'] | DIRECT_PIPELINE_INTERMEDIATE_LITERALS |
| F04R.4 row/lane accumulator | ['U-RECV-ACC-RAMP', 'U-RECV-ACC-TINY', 'U-RECV-ACC-SNAP'] | DIRECT_F04R_PIPELINE_KAT_PLUS_EXISTING_STAGE_VECTORS |
| F04R.5 transform/ring | ['case1_cold_normal', 'case2_warm_normal', 'case3_cached_drain'] | DIRECT_PIPELINE_INTERMEDIATE_LITERALS |
| F04R.6 LFE post-filter | ['case1_cold_normal', 'case2_warm_normal', 'case3_cached_drain'] | DIRECT_PIPELINE_INTERMEDIATE_LITERALS |
| F04Q mode=1 routing/output | ['U-QMF-SURROUND-RESET'] | MODE1_ONLY_NOT_F04R_PREPROCESSING |
| F04R.7 temporal exchange | ['U-RECV-TEMPORAL-3', 'U-RECV-TEMPORAL-7'] | DIRECT_F04R_PIPELINE_KAT_PLUS_EXISTING_STAGE_VECTORS |
| F04D descriptor selection/ramp | ['F04-DESCRIPTOR-CONTROL-V1 six selected-provider cases'] | DIRECT_F04D_STAGE_ONLY_NOT_CONNECTED_TO_F04R_TAIL |
| F04RB mode=0 fitting | ['F04RB-MODE0-FITTING-V1 cold g2, warm g3, cached-drain g4'] | DIRECT_F04RB_BRANCH_ONLY_NOT_FINAL_SIX |
| F04R.8 final-six tail | ['case1_cold_normal', 'case2_warm_normal', 'case3_cached_drain'] | DIRECT_F04R_FINAL_SIX_AND_COMPLETE_AFTER_STATE_KAT |

The six whole-chain vectors remain model-first assertions only and do not qualify the isolated stage. The separate continuous direct KAT computes case1 -> case2 -> case3 from actual after-state, compares intermediate outputs and the complete canonical ModelState after bytes, and is locally verified. The mapped error outcomes, including competing-fault precedence, are also locally verified. Selected provider values are identity-pinned and present.

## Minimum blockers

| blocker | gap | minimum evidence |
|---|---|---|
| f05r56-same-runner-targets | F05R56 same-runner production of target_updates remains outside this direct F04R implementation slice; direct KATs provide explicit same-generation immutable updates. | Separate F04C integration audit of F05R56 target production; direct F04R KATs remain self-contained with explicit same-generation target_updates. |

F04R-specific error labels and first-failure precedence are mapped by the approved direct receiver contract and locally verified; do not copy F04D, F04RB, or mode=1 error contracts into F04R. The remaining integration boundary is same-runner F05R56 target production, which is `NOT_RUN`. Whole receiver integration, J05/F05JC/parent commit, W04, real ADM, and product acceptance also remain `NOT_RUN`.

## Machine-readable contract

```json
{
  "blockers": [
    {
      "id": "f05r56-same-runner-targets",
      "implementation_effect": "Does not block the standalone F04R proposal contract; full callback wiring remains NOT_RUN.",
      "minimum_evidence": "Separate F04C integration audit of F05R56 target production; direct F04R KATs remain self-contained with explicit same-generation target_updates.",
      "reason": "F05R56 same-runner production of target_updates remains outside this direct F04R implementation slice; direct KATs provide explicit same-generation immutable updates."
    }
  ],
  "clock_contract": {
    "cached_drain": "All actual g=4247..4250 callbacks with Ready Type10 consume warm ModelState once; they are not synthetic flushes.",
    "dispatch_clock": "For g>=2, JOC dispatch d=g-1=s+1; dispatch remains outside this F04R map.",
    "eof": "EOF adds no new codec callback, no F04R callback, and no zero-input temporal/QMF flush.",
    "frontend_cached_drain": "g=4247..4250",
    "frontend_normal": "g=0..4246",
    "priming": "g=0/1 advance upstream component/LFE priming only; no F04R, QMF, descriptor, or F04RB call.",
    "receiver_clock": "For g>=2, receiver callback r=s=g-2; one successful callback advances receiver count once.",
    "receiver_generation": "After-state generation is r+1=g-1.",
    "source_clock": "For g>=2, source s=g-2; g=0/1 have no source-backed Type10 Ready packet for F04R.",
    "source_count": "4,249 receiver sources; s=0..4248."
  },
  "delay_and_ring_rules": {
    "first_ring": "Clamp each finite input sample to [-1,1], write at write_index, read at read_index, then advance both independent cursors by one modulo 225. Cold write=224/read=0 yields a 224-sample actual delay; this is a distinct ring owner.",
    "post_s09_delay_and_half": "After the descriptor ramp, per lane output old_delay[32] followed by current[0..1504]; set next_delay=current[1504..1536], then multiply the complete 1536-sample lane by 0.5.",
    "second_ring": "Apply a second independent ring after half scale, with the same clamp/write-before-read/cursor-modulo-225 rule as the first ring; cold write=224/read=0. Do not alias the two ring states.",
    "temporal_exchange": "index_28 is the semantic read cursor; index_32 is the semantic write cursor; stride_bytes is 4. Both cursors use the inclusive range 0..2*span; 2*span is valid and is not eagerly reduced. For N=1536, count=min(span,N), write_first=min(count,2*span-write), read_first=min(count,2*span-read); write the last count input samples with one wrap before reading count outputs from the updated buffer with one wrap, then append input[0:N-count]. next_write=write+count if count==write_first else count-write_first; next_read=read+count if count==read_first else count-read_first.",
    "transform_delay_and_blend": "For each lane and 32-sample block, emit the old 32-sample delay and replace it with the current accumulator block. With W=receiver_blend32 and a5=a6=2^-8, evaluate left=(x[j]*a5)*W[31-j], right=(x[j]*a6)*W[j], output=(left+right)*256 using the specified separate F32 operations; do not merge complementary-window products."
  },
  "error_mapping": {
    "authority": "Washed direct F04R receiver contract; these clean-room labels are not claims of native package error-code equivalence.",
    "do_not_infer_from": [
      "F04D direct outcome map",
      "F04RB mode=0 outcome map",
      "F04Q mode=1 local outcomes"
    ],
    "error_outcomes": [
      {
        "caller_action": "Return STAGE_FATAL with no preconditioned output, s09_input, final_six, after-state proposal, F04R/F04RB/F04D/J05 receipt, or commit; preserve the committed root byte-identically; do not evaluate later errors for externally visible reporting.",
        "category": "provider availability",
        "error_code": "F04R_PROVIDER_MISSING",
        "programme_state_after": "RETIRED",
        "trigger": "required selected provider is unavailable"
      },
      {
        "caller_action": "Return STAGE_FATAL with no preconditioned output, s09_input, final_six, after-state proposal, F04R/F04RB/F04D/J05 receipt, or commit; preserve the committed root byte-identically; do not evaluate later errors for externally visible reporting.",
        "category": "provider identity/shape/finite",
        "error_code": "F04R_PROVIDER_IDENTITY",
        "programme_state_after": "RETIRED",
        "trigger": "selected provider identity, shape, or finite validation fails"
      },
      {
        "caller_action": "Return STAGE_FATAL with no preconditioned output, s09_input, final_six, after-state proposal, F04R/F04RB/F04D/J05 receipt, or commit; preserve the committed root byte-identically; do not evaluate later errors for externally visible reporting.",
        "category": "lineage",
        "error_code": "F04R_LINEAGE_TARGET_UPDATES",
        "programme_state_after": "RETIRED",
        "trigger": "programme/attempt/transaction/callback/profile/generation or F05R56 receipt is missing or stale"
      },
      {
        "caller_action": "Return STAGE_FATAL with no preconditioned output, s09_input, final_six, after-state proposal, F04R/F04RB/F04D/J05 receipt, or commit; preserve the committed root byte-identically; do not evaluate later errors for externally visible reporting.",
        "category": "state shape",
        "error_code": "F04R_STATE_SHAPE",
        "programme_state_after": "RETIRED",
        "trigger": "canonical ModelState shape is invalid"
      },
      {
        "caller_action": "Return STAGE_FATAL with no preconditioned output, s09_input, final_six, after-state proposal, F04R/F04RB/F04D/J05 receipt, or commit; preserve the committed root byte-identically; do not evaluate later errors for externally visible reporting.",
        "category": "temporal cursor",
        "error_code": "F04R_TEMPORAL_CURSOR",
        "programme_state_after": "RETIRED",
        "trigger": "temporal span/stride/lane/buffer/cursor validation fails"
      },
      {
        "caller_action": "Return STAGE_FATAL with no preconditioned output, s09_input, final_six, after-state proposal, F04R/F04RB/F04D/J05 receipt, or commit; preserve the committed root byte-identically; do not evaluate later errors for externally visible reporting.",
        "category": "Type10 shape",
        "error_code": "F04R_TYPE10_SHAPE",
        "programme_state_after": "RETIRED",
        "trigger": "Type10 input is not 16 rows x 1536 F32 samples"
      },
      {
        "caller_action": "Return STAGE_FATAL with no preconditioned output, s09_input, final_six, after-state proposal, F04R/F04RB/F04D/J05 receipt, or commit; preserve the committed root byte-identically; do not evaluate later errors for externally visible reporting.",
        "category": "target shape",
        "error_code": "F04R_TARGET_SHAPE",
        "programme_state_after": "RETIRED",
        "trigger": "target updates are not 16 rows x 6 F32 values"
      },
      {
        "caller_action": "Return STAGE_FATAL with no preconditioned output, s09_input, final_six, after-state proposal, F04R/F04RB/F04D/J05 receipt, or commit; preserve the committed root byte-identically; do not evaluate later errors for externally visible reporting.",
        "category": "input finite",
        "error_code": "F04R_INPUT_NONFINITE",
        "programme_state_after": "RETIRED",
        "trigger": "a Type10 or target input is nonfinite"
      },
      {
        "caller_action": "Return STAGE_FATAL with no preconditioned output, s09_input, final_six, after-state proposal, F04R/F04RB/F04D/J05 receipt, or commit; preserve the committed root byte-identically; do not evaluate later errors for externally visible reporting.",
        "category": "numeric intermediate",
        "error_code": "F04R_NUMERIC_NONFINITE",
        "programme_state_after": "RETIRED",
        "trigger": "a post-validation numeric intermediate becomes nonfinite"
      }
    ],
    "first_failure_precedence": [
      "F04R_PROVIDER_MISSING",
      "F04R_PROVIDER_IDENTITY",
      "F04R_LINEAGE_TARGET_UPDATES",
      "F04R_STATE_SHAPE",
      "F04R_TEMPORAL_CURSOR",
      "F04R_TYPE10_SHAPE",
      "F04R_TARGET_SHAPE",
      "F04R_INPUT_NONFINITE",
      "F04R_NUMERIC_NONFINITE"
    ],
    "known_parent_effect": "Each listed error yields STAGE_FATAL; I03 retires the attempt; committed root stays byte-identical; no F04R output/proposal or receipt is accepted.",
    "negative_case_pointers": [
      {
        "expected_error": "F04R_PROVIDER_MISSING",
        "id": "missing_provider"
      },
      {
        "expected_error": "F04R_PROVIDER_IDENTITY",
        "id": "provider_identity"
      },
      {
        "expected_error": "F04R_LINEAGE_TARGET_UPDATES",
        "id": "stale_target_updates"
      },
      {
        "expected_error": "F04R_STATE_SHAPE",
        "id": "invalid_model_state"
      },
      {
        "expected_error": "F04R_TEMPORAL_CURSOR",
        "id": "invalid_temporal_cursor"
      },
      {
        "expected_error": "F04R_TYPE10_SHAPE",
        "id": "type10_shape"
      },
      {
        "expected_error": "F04R_TARGET_SHAPE",
        "id": "target_shape"
      },
      {
        "expected_error": "F04R_INPUT_NONFINITE",
        "id": "nonfinite_input"
      },
      {
        "expected_error": "F04R_NUMERIC_NONFINITE",
        "id": "numeric_nonfinite"
      }
    ]
  },
  "functional_id": "F04R",
  "implementation_prohibitions": [
    "Implement only the direct F04R receiver proposal contract; do not add a local commit or F04C parent transaction.",
    "Do not implement or rerun F04RB, F05R56, F05JC, J05, F05JL, codec, or whole-song behavior.",
    "F05JC/F05JL must not rerun F04R, F04D, or F04RB state transitions.",
    "Do not promote spatial_audio, whole receiver, W04, real ADM, or product acceptance readiness.",
    "Expected after-state and output literals are assertions only and must not seed computation."
  ],
  "implementation_readiness": "LOCALLY_VERIFIED_F04R_DIRECT_RECEIVER_ONLY",
  "lane_contract": {
    "center": "Output lane2 is C; it participates in QMF analysis history only, not mode=1 delay/FIR/synthesis.",
    "component_rows": "Rows1..15 contribute through their per-row six-lane coefficients; no extra row-to-output permutation is introduced.",
    "front": "Output lanes0/1 are L/R; they participate in QMF analysis history only, not mode=1 delay/FIR/synthesis.",
    "lfe": "Output lane3 is LFE; it receives the dedicated four-section post-filter and bypasses QMF analysis and synthesis.",
    "output_order": [
      "L",
      "R",
      "C",
      "LFE",
      "Ls",
      "Rs"
    ],
    "surround": "Output lanes4/5 are Ls/Rs; these alone receive receiver mode=1 delay, band0 FIR and QMF synthesis."
  },
  "mapping_document": "docs/implementation/RECEIVER-PIPELINE-MAP.md",
  "mapping_status": "MAPPING_COMPLETE",
  "owner_and_consumer_boundaries": {
    "F04C": "Unique executor of a full receiver callback and sole producer of selected-j05-owner-receipt.v1 after the whole callback; owns callback coordination and commit boundary.",
    "F04D": "Descriptor substage only: selects after temporal0..2/before temporal3, applies ramp after temporal3; no local commit, no J05 receipt, and no final_six input to selection.",
    "F04R": "Receiver producer contract for the ordered preprocessing, mode=1 receiver, temporal, and final_six stages; state is proposed under the spatial_audio owner.",
    "F04RB": "Independent mode=0 fitting branch with separate fitting_qmf_state; uses same-generation s09_input and descriptor after-index projection, never aliases receiver delay/FIR histories or consumes final_six.",
    "F05JC": "Downstream same-generation JOC producer consumes prepared mode=0 fitting/target/carrier inputs and projections; it must not rerun or commit F04R/F04D/F04RB state.",
    "J05_F05JL": "F04C assembles the selected J05 owner receipt; F05JL only consumes that projection and must not rerun F04R, F04D, or F04RB.",
    "writer_and_parent": "JOC writer dispatch and all parent transaction/commit actions are out of scope; no F04R-local publication or commit."
  },
  "package_identity": {
    "manifest_bytes": 76302,
    "manifest_file": "MANIFEST.json",
    "manifest_sha256": "c7cddb7d5ac073c0f410fe44be9a431cfbb21c3df469d8b43c32a4297298674a"
  },
  "persistent_owner": "spatial_audio.F04CompositionState.receiver_state: ModelState",
  "pipeline_order": [
    {
      "input_output": "Type10[16][1536] -> preconditioned[16][1536].",
      "operation": "Run each of 16 rows independently over 1536 samples in six 256-sample chunks; use alpha, beta, strict tiny threshold, and fixed scale 0.25; scalar carries across chunks and callbacks.",
      "order": 1,
      "owner": "F04R producer, called once by F04C for this successful receiver callback",
      "stage": "F04R.3 scalar preconditioning",
      "state_fields": [
        "ModelState.filter_state"
      ]
    },
    {
      "input_output": "preconditioned rows plus target_updates[g] -> accumulator[6][1536].",
      "operation": "Install only supplied same-g target values; clear six 1536-sample accumulators; process 48 ordered 32-sample blocks, row0..15 then lane0..5, using the per-row/per-lane ramp state.",
      "order": 2,
      "owner": "F04R producer",
      "stage": "F04R.4 target installation and row/lane accumulation",
      "state_fields": [
        "ModelState.row_states"
      ]
    },
    {
      "input_output": "accumulator[6][1536] -> six-lane pre-LFE-filter candidate[6][1536].",
      "operation": "Per lane emit the prior 32-sample delay, save the current accumulator block, perform the ordered two-ended N32 blend and fixed 256 scaling, then clamp/write/read/advance the independent 225-sample first ring.",
      "order": 3,
      "owner": "F04R producer",
      "stage": "F04R.5 transform delay, PG01 blend, first ring",
      "state_fields": [
        "ModelState.transform_delay",
        "ModelState.first_ring"
      ]
    },
    {
      "input_output": "pre-LFE-filter candidate lane3 -> filtered lane3; lanes0/1/2/4/5 pass unchanged; combine as s09_input[6][1536].",
      "operation": "Process only lane3 as six ordered 256-sample chunks through four coefficient sections in pair order; carry the four section states across chunks and callbacks; do not add a tiny-history clear.",
      "order": 4,
      "owner": "F04R producer",
      "stage": "F04R.6 LFE post-filter",
      "state_fields": [
        "ModelState.post_filter_state"
      ]
    },
    {
      "input_output": "s09_input[6][1536] with lane3 already post-filtered -> QMF analysis histories and updated Ls/Rs PCM scratch.",
      "operation": "Analyze lanes0/1/2/4/5 as 24 ordered 64-sample slots; only lanes4/5 apply ten-slot delay, mode=1 phase transform, 21-tap band0 FIR and synthesis. Lane3 bypasses QMF.",
      "order": 5,
      "owner": "F04R producer through the separate F04Q stage",
      "stage": "F04Q receiver analysis and mode=1 output",
      "state_fields": [
        "ModelState.analysis_history",
        "ModelState.qmf_delay_history",
        "ModelState.qmf_fir_history",
        "ModelState.synthesis_history"
      ]
    },
    {
      "input_output": "QMF/post-filter scratch -> post-temporal0..2 scratch for descriptor selection.",
      "operation": "Run three independent double-span exchanges in this order: temporal0 span1217 lanes[0,1,2], temporal1 span63 lanes[0,1,2,4,5], temporal2 span900 lane[3].",
      "order": 6,
      "owner": "F04R producer",
      "stage": "F04R.7 temporal0 then temporal1 then temporal2",
      "state_fields": [
        "ModelState.temporal_states[0..3]"
      ]
    },
    {
      "input_output": "post-temporal0..2 scratch -> selected index/control proposal carried across temporal3.",
      "operation": "Select from post-temporal0..2 scratch and descriptor before-state; peak reads lanes[0,1,2,4,5], excluding LFE lane3. This phase returns a same-callback selection/proposal, not final_six.",
      "order": 7,
      "owner": "F04D substage, executed within the F04C-owned callback",
      "stage": "F04D.3 descriptor selection before temporal3",
      "state_fields": [
        "ModelState.descriptor_state"
      ]
    },
    {
      "input_output": "post-temporal0..2 six-lane scratch -> post-temporal3 scratch.",
      "operation": "Run the fourth exchange over all six lanes with span256 after descriptor selection; never combine temporal0..3 into one buffer.",
      "order": 8,
      "owner": "F04R producer",
      "stage": "F04R.7 temporal3",
      "state_fields": [
        "ModelState.temporal_states[3]"
      ]
    },
    {
      "input_output": "post-temporal3 scratch -> descriptor-gained six-lane output.",
      "operation": "Apply the selected target to all six lanes after temporal3; the first 256 samples share the callback ramp and samples256..1535 use target gain.",
      "order": 9,
      "owner": "F04D substage, executed within the F04C-owned callback",
      "stage": "F04D.4 descriptor gain ramp",
      "state_fields": [
        "ModelState.descriptor_state"
      ]
    },
    {
      "input_output": "descriptor-gained six-lane output -> delayed and half-scaled six-lane output.",
      "operation": "Per lane prepend the prior 32-sample tail to current[0..1504], save current[1504..1536] as the next tail, then multiply each sample by 0.5.",
      "order": 10,
      "owner": "F04R producer",
      "stage": "F04R.8 post-S09 delay and half scale",
      "state_fields": [
        "ModelState.post_s09_delay"
      ]
    },
    {
      "input_output": "delayed half-scaled lanes -> final_six[6][1536].",
      "operation": "Apply the independent second ring with the same clamp, write-before-read, and 225-slot cursor rules as the first ring.",
      "order": 11,
      "owner": "F04R producer; output remains subordinate to the enclosing F04C callback",
      "stage": "F04R.8 second ring and final_six",
      "state_fields": [
        "ModelState.second_ring"
      ]
    }
  ],
  "programme_root": "spatial_audio",
  "provider_bindings": [
    {
      "bytes": 20325,
      "fields": [
        "preconditioning_alpha",
        "preconditioning_beta",
        "preconditioning_tiny_strict",
        "accumulator_magnitude_threshold",
        "accumulator_delta_threshold",
        "accumulator_smoothing",
        "lfe_post_filter_4x5",
        "qwin_synthesis_640",
        "band0_real_mode1_21",
        "band0_imag_mode1_21",
        "descriptor_threshold_127",
        "descriptor_level_offset",
        "descriptor_level_linear",
        "descriptor_level_quadratic",
        "descriptor_headroom_step",
        "descriptor_result_scale",
        "descriptor_rise"
      ],
      "file": "parameters/qmf-receiver-functional-v1.json",
      "sha256": "30e63cc110b63dd88118ad37f02750a39167bd01279a3b40853afdf3151b552c"
    },
    {
      "bytes": 2704,
      "fields": [
        "receiver_blend32[32] as Q15 values, converted to F32 exactly for F04R.5"
      ],
      "file": "parameters/window-q15.json",
      "sha256": "a12f9cbf91f1cf540c2eae77e4d236970defa70409f00ddf62ba87bfeafbb130"
    },
    {
      "bytes": 6983,
      "fields": [
        "QMF FFT roots and rotation used by the separate F04Q stage"
      ],
      "file": "parameters/qmf-mathematical-fft-v1.json",
      "sha256": "e8d3fb1514a5ffd857740222eacc8995208a29c3447e48c1b63083af368c0431"
    },
    {
      "bytes": 4861,
      "fields": [
        "PG03 gain[128] for the F04D ramp; its mathematical threshold table is not the selected F04D lookup threshold"
      ],
      "file": "parameters/descriptor-mathematical.json",
      "sha256": "a493fbdaf2dab30e940cae9976ed3305f3af083e5614ab0cbd6889c71a02b197"
    }
  ],
  "readiness_boundary": {
    "F04R_direct_contract": "LOCALLY_VERIFIED_F04R_DIRECT_STAGE_ONLY",
    "F04R_direct_stage_execution": "LOCALLY_VERIFIED_DIRECT_KAT_ONLY",
    "F04R_error_outcomes": "LOCALLY_VERIFIED_DIRECT_NEGATIVE_AND_PRECEDENCE_CASES",
    "F04R_final_six_direct_kat": "LOCALLY_VERIFIED",
    "F04R_production_callback_integration": "NOT_RUN",
    "J05_F05JC_and_parent_commit": "OUT_OF_SCOPE_AND_NOT_RUN",
    "W04_real_ADM_and_product_acceptance": "NOT_RUN",
    "mapping": "MAPPING_COMPLETE",
    "same_runner_F05R56_target_production": "NOT_RUN",
    "temporal_state_field_crosswalk": "MAPPED_EXPLICITLY_BY_APPROVED_F04R_SPEC",
    "whole_spatial_receiver": "NOT_RUN"
  },
  "specifications": [
    {
      "bytes": 8332,
      "clauses": [
        "F04R.1",
        "F04R.2",
        "F04R.3",
        "F04R.4",
        "F04R.5",
        "F04R.6",
        "F04R.7",
        "F04R.8"
      ],
      "file": "specs/04-receiver-state-and-routing.md",
      "sha256": "74b9b01537eb76a37582535b84426138ee8c8ecdca3486f9f43d0504c7c26306"
    },
    {
      "bytes": 9103,
      "clauses": [
        "F04C.3",
        "F04C.4",
        "F04C.5",
        "F04C.6",
        "F04C.7",
        "F04C.8"
      ],
      "file": "specs/04-whole-spatial-chain-composition.md",
      "sha256": "92325b12128023001e1f3fd4696dcec68bbd47a46acdb7886781b411a10c5d59"
    },
    {
      "bytes": 4504,
      "clauses": [
        "Callback split",
        "J05 receipt",
        "Source timing"
      ],
      "file": "specs/00-selected-owner-projection-and-j05.md",
      "sha256": "9ff01aca7b6be6e89511e4d493f9fc9e661a50e2f098065f4a32c3fed777542f"
    }
  ],
  "state_ownership": {
    "canonical_encoding": "The full spatial_audio root uses the package-bound encode_node-v1 schema and field order; F04R proposes one complete ModelState after, while only the enclosing F04C/spatial_audio transaction can commit.",
    "canonical_schema_bytes": 226344,
    "canonical_schema_file": "test-vectors/F04-WHOLE-SPATIAL-STATE-SCHEMA.json",
    "canonical_schema_sha256": "4fdab9f591821d7c50e7daca17bc98d4a5b0616e9cb45b5937cae47b26ed657c",
    "cold_provider": "F04R.1 cold values plus the package-bound whole-state schema; no separate runtime cold-state provider. Reconstruct the complete owner cold root, including both rings and temporal cursors.",
    "commit_authority": "Enclosing spatial_audio callback transaction only; no F04R, F04D, or F04RB local commit.",
    "eof_drain": "Run each actual cached-drain callback with Ready Type10 once through F04R; EOF adds no synthetic F04R callback or codec flush.",
    "fatal_behavior": "A failed callback returns no accepted ModelState after or final_six; I03 applies STAGE_FATAL and retires the programme; the parent committed root remains unchanged.",
    "fields": [
      {
        "cold": "all positive zero",
        "field": "filter_state",
        "schema": "16 F32 positive-zero scalars"
      },
      {
        "cold": "all F32 positive zero; remaining all U32 zero",
        "field": "row_states",
        "schema": "16 RowState values; each current/target/slope is 6 F32 and remaining is 6 U32"
      },
      {
        "cold": "all positive zero",
        "field": "transform_delay",
        "schema": "6 lanes x 32 F32"
      },
      {
        "cold": "words positive zero; write=224, read=0",
        "field": "first_ring",
        "schema": "6 RingState values; words=225 F32, write_index/read_index are I64LE"
      },
      {
        "cold": "all positive zero",
        "field": "post_filter_state",
        "schema": "4 sections x 2 F32"
      },
      {
        "cold": "all positive zero",
        "field": "analysis_history",
        "schema": "5 rows [L,R,C,Ls,Rs] x 9 x 64 F32"
      },
      {
        "cold": "all positive zero",
        "field": "qmf_delay_history",
        "schema": "2 rows [Ls,Rs] x 10 x 128 F32"
      },
      {
        "cold": "all positive zero",
        "field": "qmf_fir_history",
        "schema": "2 rows [Ls,Rs] x 40 F32"
      },
      {
        "cold": "all positive zero",
        "field": "synthesis_history",
        "schema": "2 rows [Ls,Rs] x 576 F32"
      },
      {
        "cold": "F04R.1 gives logical read=span/write=0 and two positive-zero buffers per lane; the approved F04R semantic crosswalk maps index_28=read and index_32=write; temporal4 remains unchanged in the selected route",
        "field": "temporal_states",
        "schema": "TemporalState(span:I64LE,stride_bytes:I64LE=4,index_28:I64LE semantic read,index_32:I64LE semantic write,buffers); temporal0=(span1217,3 lanes), temporal1=(span63,5 lanes), temporal2=(span900,1 lane), temporal3=(span256,6 lanes), reserved temporal4=(span896,15 lanes) unchanged."
      },
      {
        "cold": "tail_peak=+0, hold=0, smoothed=+0, exponent=2, reference=R32(2/48), gain=1/16",
        "field": "descriptor_state",
        "schema": "DescriptorState(tail_peak:F32, hold_count:I64LE, smoothed:F32, exponent:I64LE, reference:F32, gain:F32)"
      },
      {
        "cold": "all positive zero",
        "field": "post_s09_delay",
        "schema": "6 lanes x 32 F32"
      },
      {
        "cold": "words positive zero; write=224, read=0",
        "field": "second_ring",
        "schema": "6 RingState values; words=225 F32, write_index/read_index are I64LE"
      },
      {
        "cold": "diagnostic/reserved sentinel only; not consumed by the selected route",
        "field": "target_coefficients",
        "schema": "None in the canonical schema"
      }
    ],
    "owner": "spatial_audio.F04CompositionState.receiver_state: ModelState",
    "receipt_type": "No F04R-local receipt. F04C is the unique whole-callback executor and later selected-J05 receipt producer.",
    "reset_authority": "Programme-origin full spatial_audio reset reconstructs all declared ModelState fields from the approved cold schema; no partial final-six-only reset.",
    "schema_pointer": "#/whole_state_by_boundary/cold/fields/6/schema",
    "stage_authority": "F04R.1-.8 define the receiver route; F04D owns descriptor formulas as a substage; F04C alone executes the complete callback.",
    "temporal_state_serialization": {
      "cold_semantics": "index_28=span (read); index_32=0 (write); stride_bytes=4.",
      "entry_fields": [
        {
          "encoding": "I64LE",
          "name": "span"
        },
        {
          "encoding": "I64LE",
          "name": "stride_bytes"
        },
        {
          "encoding": "I64LE",
          "name": "index_28"
        },
        {
          "encoding": "I64LE",
          "name": "index_32"
        },
        {
          "encoding": "lane tuple; two span-sized F32 buffers per lane",
          "name": "buffers"
        }
      ],
      "instances": [
        {
          "buffer_bytes_per_lane": 9736,
          "lanes": 3,
          "name": "temporal0",
          "span": 1217
        },
        {
          "buffer_bytes_per_lane": 504,
          "lanes": 5,
          "name": "temporal1",
          "span": 63
        },
        {
          "buffer_bytes_per_lane": 7200,
          "lanes": 1,
          "name": "temporal2",
          "span": 900
        },
        {
          "buffer_bytes_per_lane": 2048,
          "lanes": 6,
          "name": "temporal3",
          "span": 256
        },
        {
          "buffer_bytes_per_lane": 7168,
          "lanes": 15,
          "name": "temporal4",
          "route_status": "reserved and unchanged",
          "span": 896
        }
      ],
      "schema_pointer": "#/whole_state_by_boundary/cold/fields/6/schema/fields/9/schema",
      "semantic_crosswalk": "index_28 is the semantic read cursor; index_32 is the semantic write cursor; stride_bytes is 4. Both cursors use the inclusive range 0..2*span; 2*span is valid and is not eagerly reduced. For N=1536, count=min(span,N), write_first=min(count,2*span-write), read_first=min(count,2*span-read); write the last count input samples with one wrap before reading count outputs from the updated buffer with one wrap, then append input[0:N-count]. next_write=write+count if count==write_first else count-write_first; next_read=read+count if count==read_first else count-read_first.",
      "semantic_crosswalk_gap": "",
      "semantic_crosswalk_status": "MAPPED_EXPLICITLY_BY_APPROVED_F04R_SPEC"
    }
  },
  "status": "LOCALLY_VERIFIED_F04R_DIRECT_RECEIVER_ONLY__WHOLE_RECEIVER_NOT_RUN",
  "task_id": "F04R-PREPROCESSING-FINAL-SIX-MAPPING-AUDIT",
  "type10_input": {
    "callback_index": "FrontendCallbackIndex g; never substitute receiver count r.",
    "lineage": "Preserve Type10 packet/receipt, parent programme/attempt/transaction, g, explicit phase, selected profile, and the enclosing spatial_audio before-root as one F04C callback.",
    "ready_boundary": "F04C consumes the E4 Ready packet and receipt pair only for g>=2; g=0/1 are Type10 priming and do not call F04R.",
    "receiver_s09_boundary": "F04R.6 post-filters lane3 first; that filtered lane3 plus the other five unchanged F04R.5 lanes form the same-generation s09_input. F04RB consumes this exact pre-fitting s09_input independently; F04Q then analyzes lanes0/1/2/4/5 and applies mode1 output only to lanes4/5.",
    "row_order": "row0 is the delayed direct-LFE source for source s=g-2; rows1..15 are component15[g][0..14] for the current callback g.",
    "shape": "16 rows x 1536 F32 samples.",
    "target_updates": "Required contract input is same-callback target_updates[g] from the F05R56 target-owner receipt; same-runner producer wiring is NOT_RUN. Update only each supplied row.target[6], retaining current/slope/remaining; rows without updates retain their old target."
  },
  "vector_coverage": {
    "approved_supplement_identity": {
      "api_schema": {
        "bytes": 988,
        "file": "API-SCHEMA.json",
        "sha256": "1054e37a3c919fefc0d46398a460b3a96b09247ab05af6ce0533f769d1663c59"
      },
      "current_package_manifest_sha256": "c7cddb7d5ac073c0f410fe44be9a431cfbb21c3df469d8b43c32a4297298674a",
      "current_rebind_manifest_sha256": "30ac40e7ee6fb2cf23117b112b7095d2266e250e21781c509c1c34bdad9f9b0d",
      "current_rebind_sha256": "8e359eb3f09bd39e6732ce89655786cd3920c3019a0ba6bee84b5ecfab9e2326",
      "fresh_actual_builder_sha256": "36e7dfcfd3881dc3479612175a2a663334760ffc3f738f5f1d0cdf582af97b4a",
      "historical_package_manifest_sha256": "e6279853ee0812c46f788aa4112e81c471ccf6307e6ec1ca45278bcd56a91e70",
      "historical_role": "historical_generation_provenance_only",
      "identity": "f04r-receiver-final-six.v1-current-c7-rebind-chain",
      "manifest_sha256": "075b90e684c4edc55eb7c8d78cf0c60161ad68d2d585c82a7b7da614cb5dbc3c",
      "specification": {
        "bytes": 7780,
        "file": "SPEC.md",
        "sha256": "2e76a75a549d5963ee38ccdfc605fa81840b7e944084a288c72076739a136085"
      }
    },
    "complete_after_state_literal": "PRESENT_CANONICAL_MODELSTATE_AFTER",
    "direct_cases": [
      "case1_cold_normal",
      "case2_warm_normal",
      "case3_cached_drain"
    ],
    "direct_fixture_inventory": {
      "actual_literals": {
        "bytes": 19198,
        "file": "F04R-DIRECT-ACTUAL-LITERALS.json",
        "sha256": "b1f4defcd303c5fd89c05b4c13af5a1a6f7cb1525157b314b2ba63e197baaf41"
      },
      "binary_count": 27,
      "directory": "crates/jocforge-domain/tests/fixtures/receiver_pipeline",
      "file_count": 29,
      "input_vectors": {
        "bytes": 3022,
        "file": "F04R-DIRECT-INPUT-VECTORS.json",
        "sha256": "77cec5f7e624ad150dbbeeee2e1a43b78bd2ed5d00726a7b35b9302871e53ee3"
      }
    },
    "direct_full_pipeline_kat": "PRESENT_FOR_DIRECT_F04R_RECEIVER_ONLY",
    "literal_input_policy": "F04R-DIRECT-INPUT-VECTORS.json and its Type10/target binaries are the only input seeds. Actual literals and expected after-state binaries are post-execution assertions only and must never seed this or a later case.",
    "negative_cases": [
      {
        "expected_error": "F04R_PROVIDER_MISSING",
        "id": "missing_provider"
      },
      {
        "expected_error": "F04R_PROVIDER_IDENTITY",
        "id": "provider_identity"
      },
      {
        "expected_error": "F04R_LINEAGE_TARGET_UPDATES",
        "id": "stale_target_updates"
      },
      {
        "expected_error": "F04R_STATE_SHAPE",
        "id": "invalid_model_state"
      },
      {
        "expected_error": "F04R_TEMPORAL_CURSOR",
        "id": "invalid_temporal_cursor"
      },
      {
        "expected_error": "F04R_TYPE10_SHAPE",
        "id": "type10_shape"
      },
      {
        "expected_error": "F04R_TARGET_SHAPE",
        "id": "target_shape"
      },
      {
        "expected_error": "F04R_INPUT_NONFINITE",
        "id": "nonfinite_input"
      },
      {
        "expected_error": "F04R_NUMERIC_NONFINITE",
        "id": "numeric_nonfinite"
      }
    ],
    "stage_coverage": [
      {
        "classification": "DIRECT_PIPELINE_INTERMEDIATE_LITERALS",
        "direct_cases": [
          "case1_cold_normal",
          "case2_warm_normal",
          "case3_cached_drain"
        ],
        "stage": "F04R.3 scalar preconditioning"
      },
      {
        "classification": "DIRECT_F04R_PIPELINE_KAT_PLUS_EXISTING_STAGE_VECTORS",
        "direct_cases": [
          "U-RECV-ACC-RAMP",
          "U-RECV-ACC-TINY",
          "U-RECV-ACC-SNAP"
        ],
        "stage": "F04R.4 row/lane accumulator"
      },
      {
        "classification": "DIRECT_PIPELINE_INTERMEDIATE_LITERALS",
        "direct_cases": [
          "case1_cold_normal",
          "case2_warm_normal",
          "case3_cached_drain"
        ],
        "stage": "F04R.5 transform/ring"
      },
      {
        "classification": "DIRECT_PIPELINE_INTERMEDIATE_LITERALS",
        "direct_cases": [
          "case1_cold_normal",
          "case2_warm_normal",
          "case3_cached_drain"
        ],
        "stage": "F04R.6 LFE post-filter"
      },
      {
        "classification": "MODE1_ONLY_NOT_F04R_PREPROCESSING",
        "direct_cases": [
          "U-QMF-SURROUND-RESET"
        ],
        "stage": "F04Q mode=1 routing/output"
      },
      {
        "classification": "DIRECT_F04R_PIPELINE_KAT_PLUS_EXISTING_STAGE_VECTORS",
        "direct_cases": [
          "U-RECV-TEMPORAL-3",
          "U-RECV-TEMPORAL-7"
        ],
        "stage": "F04R.7 temporal exchange"
      },
      {
        "classification": "DIRECT_F04D_STAGE_ONLY_NOT_CONNECTED_TO_F04R_TAIL",
        "direct_cases": [
          "F04-DESCRIPTOR-CONTROL-V1 six selected-provider cases"
        ],
        "stage": "F04D descriptor selection/ramp"
      },
      {
        "classification": "DIRECT_F04RB_BRANCH_ONLY_NOT_FINAL_SIX",
        "direct_cases": [
          "F04RB-MODE0-FITTING-V1 cold g2, warm g3, cached-drain g4"
        ],
        "stage": "F04RB mode=0 fitting"
      },
      {
        "classification": "DIRECT_F04R_FINAL_SIX_AND_COMPLETE_AFTER_STATE_KAT",
        "direct_cases": [
          "case1_cold_normal",
          "case2_warm_normal",
          "case3_cached_drain"
        ],
        "stage": "F04R.8 final-six tail"
      }
    ],
    "state_chain_semantics": {
      "cases": [
        {
          "before_state_source": "fresh_rcold_constructor",
          "id": "case1_cold_normal"
        },
        {
          "before_state_source": "fresh_computed_case1_after",
          "id": "case2_warm_normal"
        },
        {
          "before_state_source": "fresh_computed_case2_after",
          "id": "case3_cached_drain"
        }
      ],
      "execution": "one_continuous_chain",
      "expected_after_role": "post_execution_assertion_only",
      "expected_after_used_as_next_before_seed": false
    },
    "unit_vector_file": {
      "bytes": 1503142,
      "file": "test-vectors/unit/vectors.json",
      "sha256": "a14dd6c3700fae1765399c72c816dd3146d57ab7dd026b375efb6aed7408cfa8"
    },
    "whole_chain_file": {
      "bytes": 260481,
      "cases": 6,
      "file": "test-vectors/F04-WHOLE-SPATIAL-CHAIN-VERIFICATION.json",
      "qualification": "MODEL_FIRST_ASSERTIONS_ONLY; not isolated F04R execution, not a full final_six direct KAT, and not product acceptance",
      "sha256": "ffb1dea3b1e69383312da4059b9d8ddabb4366ed8fc22e7537f750ba041f2d63"
    }
  },
  "whole_callback_executor": "F04C"
}
```
