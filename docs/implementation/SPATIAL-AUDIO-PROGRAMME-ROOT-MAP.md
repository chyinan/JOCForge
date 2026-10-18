# spatial_audio programme-root mapping

## Status and scope

`programme_roots.spatial_audio` maps to the complete approved-package `F04CompositionState` cold schema. The cold specification is `RESOLVED_BY_F04_COLD_SUPPLEMENT`; P1.3-E1 is ready for its cold storage/staging implementation, which is not yet locally verified. No DSP, provider validator, render producer, or CallbackTransaction coordinator was implemented or validated.

The complete spatial_audio integration remains blocked on F04-specific P1.4 provider contracts, the post-window packet producer/fanout contract, and the enclosing callback transaction. P1.3-C remains `CALLBACK_CONTRACT_APPROVED_WAITING_OWNER_ROOTS` because the other callback roots and g>=2 source-stage roots are still missing.

The cold supplement manifest is `f04-spatial-audio-cold-state.v1`, SHA-256 `77ad0562fcd4efa1e8fe37cc1dc7219570b49b9b332cca1f1273ccda2e2cef9c`. Its 12 file identities were verified without opening `research_basis`; the 813,733-byte KAT was only hashed opaquely for its manifest identity. It may be compared only after a fresh typed constructor and project encoder produce bytes.

## Identity and GTC binding

The approved-package `MANIFEST.json` identity is SHA-256 `c7cddb7d5ac073c0f410fe44be9a431cfbb21c3df469d8b43c32a4297298674a`.

| Role | File and pointer | Identity / meaning |
|---|---|---|
| Programme slot | `test-vectors/GLOBAL-TRANSACTION-PROGRAMME-STATE-SCHEMA.json#/properties/owners/properties/spatial_audio` | `$ref: #/$defs/ownerState` |
| GTC wrapper | `#/$defs/ownerState` | `owner_id, generation, state_sha256, cold_sha256, state_ref` |
| Root schema candidate | `test-vectors/F04-WHOLE-SPATIAL-STATE-SCHEMA.json#/whole_state_by_boundary/cold` | `F04CompositionState`, nine declared fields; file SHA-256 `4fdab9f591821d7c50e7daca17bc98d4a5b0616e9cb45b5937cae47b26ed657c`; cold preimage not uniquely bound |
| Schema semantic identity | same schema file, top-level `sha256` | `e1ef560564678a04494a9a563b1e431582fd0a48c8524ff4e4b47cf7a880655c`; this is not a state digest |
| Expected cold owner digest | `test-vectors/F04-WHOLE-SPATIAL-CHAIN-VERIFICATION.json#/reset_check/cold_state_sha256` | `35cfac927bbdca64f28954844938fe2936531f59f049dfca911c44d3a38b0b5e`; expected identity only, not a reconstructed canonical byte preimage |

The candidate `state_ref` value is `test-vectors/F04-WHOLE-SPATIAL-STATE-SCHEMA.json#/whole_state_by_boundary/cold`. The GTC slot and schema identity are mapped. The expected `cold_sha256` remains a target until the typed constructor and encoder reproduce it locally. The current `selected_overlap_projection` is only a cross-owner projection; `selected-j05-owner-receipt.v1` is a per-stage receipt. Neither is the committed programme root or a substitute for its complete state digest.

The package F04 chain result is `PASS_CONSUMER_SIDE_COMPOSITION`, limited to the installed48 consumer boundary. Raw38-to-installed48, dynamic matrix/destination production, the complete callback transaction, real ADM execution, and perceptual acceptance remain `OPEN` or `NOT_RUN`; this root map does not upgrade those claims.

## Exact root field order and cold state

The field order is the declared `F04CompositionState` order. Each `schema_pointer` below is relative to the F04 schema file's `#/whole_state_by_boundary/cold` node.

| # | Field / schema pointer | Type and cold value |
|---:|---|---|
| 0 | `callback_index` / `fields/0` | `I64LE`, cold `0`; runtime domain is the typed `FrontendCallbackIndex` and must join the enclosing callback. |
| 1 | `retired` / `fields/1` | Boolean `U8`, cold `false`; F04-local guard only. GTC programme status/fatal remains owned by the callback coordinator. |
| 2 | `component_state` / `fields/2` | `ComponentAudioState`: overlap `F32[15][1536]` and source ring `F32[15][1472]`, all positive zero. |
| 3 | `component_parameter_state` / `fields/3` | `ComponentFilterParameterState`: `peak=+0`, `previous_gain=1`, `hold=0`, `envelope=1`, `smoothed_gain=1`, `target_gain=1`, `output_level=2^-8`, `blocks_processed=0`. |
| 4 | `component_filter_history` / `fields/4` | `T15<B256>` = `F32[15][64]`, all positive zero. |
| 5 | `lfe_state` / `fields/5` | `LfeDelayRingState`: `write_index=0`, `read_index=1`, `rows=[None,None,None]`, `input_frame_indexes=[None,None,None]`, `model_samples_consumed=0`. The two `None` arrays are sentinels and must not be zero-filled. |
| 6 | `receiver_state` / `fields/6` | `ModelState`, recursively pinned below. Numeric/sample histories use the specified cold constructors; ring indices are `write=224/read=0`; `target_coefficients=None`. Temporal slot 4 remains cold and unconsumed. Its five temporal nodes use `stride_bytes=4`, `index_28=read=span`, `index_32=write=0`, with lane-major raw bytes for `2*span` positive-zero F32 values per lane. |
| 7 | `fitting_qmf_state` / `fields/7` | `QmfWindowState`, recursively pinned below. Target/carrier windows and mode-0 fitting delay/FIR histories are distinct from receiver mode-1 audio histories. `target_temporal` is a separate span-896 node with the same scalar rule and its own buffers. |
| 8 | `receiver_callback_count` / `fields/8` | `I64LE`, cold `0`; typed as `ReceiverCallbackCount`, incrementing only for `g>=2` callbacks with Type10. |

The exact recursive shape for `receiver_state` is:

- `filter_state=T16<B4>`; `row_states=T16<RowState>`, with `current`, `target`, `slope`, and `remaining` each `T6<B4>`; `transform_delay=T6<B128>`.
- `first_ring=T6<RingState>` where each ring is `{words:B900, write_index:I64LE, read_index:I64LE}`; `post_filter_state=B32`.
- `analysis_history=T5<T9<B256>>`; `qmf_delay_history=T2<T10<B512>>`; `qmf_fir_history=T2<B160>`; `synthesis_history=T2<B2304>`.
- `temporal_states=T5<TemporalState>` with ordered `(span, buffers)` pairs `(1217,T3<B9736>)`, `(63,T5<B504>)`, `(900,T1<B7200>)`, `(256,T6<B2048>)`, `(896,T15<B7168>)`; each also has its declared integer stride/index fields.
- `descriptor_state=DescriptorState` in declared order; `post_s09_delay=T6<B128>`; `second_ring=T6<RingState<B900,I64LE,I64LE>>`; `target_coefficients=None`.

The exact recursive shape for `fitting_qmf_state` is:

- `target_temporal=TemporalState(..., buffers=T15<B7168>)`;
- `target_history=T15<T9<B256>>`;
- `previous_targets=T15<T24<B512>>`;
- `previous_carriers=T5<T24<B512>>`;
- `fitting_delay=T5<T10<B512>>`;
- `fitting_fir=T5<B160>`.

The supplement resolves the previous ambiguity for all six temporal nodes: `stride_bytes=4`, `index_28=span`, `index_32=0`, with `2*span` positive-zero F32 values in each lane-major raw-byte buffer. The two span-896 nodes have equal initial contents but remain independent objects and buffers.

`F04_STATE_NODE_V1` uses `N`, `B`, `I`, `F`, `Z`, `T`, and `D` tags. `D` encodes an ASCII class name and declaration-ordered ASCII fields with U16LE lengths; `T` and `B` lengths are U64LE; `I` is signed I64LE; `F` is finite R32 then IEEE-754 binary32 little-endian; and `Z` is U8 0/1. `None`, bool, integer, float, and bytes remain distinct. The grammar has no extra D-body length, padding, header, terminator, checksum, or digest.

Cold-state provider kind is `GENERATED_PROVIDER`: no runtime parameter file constructs the root. The complete constructor is now specified; local byte reconstruction and digest verification remain implementation acceptance checks.

## Ownership, lifecycle, and receipts

| Concern | Authority / contract |
|---|---|
| Persistent `spatial_audio` fields | F04C component overlap/ring/filter and parameter state; direct-LFE ring and source generations; receiver `ModelState`; `QmfWindowState` target/carrier windows and mode-0 fitting histories. |
| Other persistent owners | `spatial_control` owns raw/previous PCM, previous-current48, installed48 producer histories, diffusion instances/outputs, targets, assignments, markers, and matrix. F05R56 owns reader/target histories. F05JL owns temporal, geometry, fit, retry, Huffman, and ID14 writer state; it consumes the F04C receipt and does not rerun or own F04R/F04RB histories. F13 owns metadata queues/timer/EMDF assembly; F15 owns source publication/output state. |
| Callback-local only | Prefix row accumulators/outputs; `matrix[g]`, destinations, `installed48[g]` input view, current LFE input, Type10, final six, target/carrier output windows, and receipts. They are not members of `F04CompositionState`. |
| Callback clock | `callback_index: FrontendCallbackIndex` must match the parent `CallbackTransactionId` and `FrontendCallbackIndex`. It is not an independent main clock. No public API may interchange it with `ReceiverCallbackCount`, source, dispatch, or AU indices. |
| Receiver clock | For `g<2`, emit priming and advance component/LFE only. For `g>=2`, `r=g-2`; advance receiver/QMF exactly once and increment `receiver_callback_count`. First valid receiver generation is `g=2,r=0`. |
| Stage / local receipt | F04C executes component, LFE, receiver, and both QMF owner transitions once. It emits a priming result for `g<2`, otherwise `selected-j05-owner-receipt.v1` with the stage lineage described by the approved selected-owner contract. This local receipt is not the GTC owner receipt. |
| Single owner / linear capabilities | One `SpatialAudioMachine` owns the committed root. Planned `SpatialAudioPrepared` and `SpatialAudioStaged` are non-Clone/non-Copy and consumed by the next stage or parent commit/abort. No second owner or independently callable local commit is allowed. |
| Commit and generation | Only the enclosing GTC `CallbackTransactionCoordinator` can replace the committed owner. `OwnerReceipt<SpatialAudio>` has the eight fields pinned by `#/$defs/ownerReceipt`; owner generation starts at 0 and increments once for each committed CALLBACK. Rejected/aborted callbacks preserve before generation and digest. |
| Reset | Programme-origin full restart reconstructs this entire cold root and invalidates staged tokens. No resume is allowed within a retired attempt. |
| EOF / drains | Process every actual callback, including the four upstream cached drains. Component/LFE advance each callback; receiver/QMF advance only when `g>=2`. Do not add a synthetic callback or QMF all-zero flush. |
| Fatal | F04 state/order/input/profile/numeric failures produce no normal output or committed after-state. The parent abort preserves all owner before snapshots and owns programme retirement; there is no F04-local top-level fatal receipt. |

## Runtime provider inventory and prerequisites

The following are algorithm providers, not cold-root byte sources. Every listed file identity is recorded in the machine map and boundary-checked against the approved package manifest. The required F04-specific P1.4 identity/shape/permission validators do not yet exist; the already verified P1.4-C validators are for CurrentFeatures providers only.

| Provider | Kind and identity | Runtime shape / delivery boundary | Status |
|---|---|---|---|
| `parameters/window-q15.json` | `GENERATED`, 2704 bytes, SHA-256 `a12f9cbf91f1cf540c2eae77e4d236970defa70409f00ddf62ba87bfeafbb130` | `receiver_blend32` and `component_kernel64` Q15 arrays; formula-generated F32LE views. No private key. | F04 P1.4 validator not implemented. |
| `parameters/component-parameter-mathematical-v1.json` | `GENERATED`, 1077 bytes, SHA-256 `5d0fec6f62a33eeef5c64016760fcbe94a1872a564b6d0e03f959c95c37eb0f5` | Eight Q15 integers and eight F32 words for 48 kHz / 64-sample blocks. No source table copied. Internal mathematical generation; file says implementation readiness `NO`. | F04 P1.4 validator not implemented. |
| `parameters/qmf-receiver-functional-v1.json` | `FILE`, 20325 bytes, SHA-256 `30e63cc110b63dd88118ad37f02750a39167bd01279a3b40853afdf3151b552c` | 829 array F32 words plus 12 scalar F32 words: QWIN synthesis 640; selected band0 real/imag 21 each; LFE postfilter 20; descriptor thresholds 127. Authorized for internal isolated use; no secret, source, raw table, compiled image, or address. | F04 P1.4 validator not implemented. |
| `parameters/qmf-mathematical-fft-v1.json` | `GENERATED`, 6983 bytes, SHA-256 `e8d3fb1514a5ffd857740222eacc8995208a29c3447e48c1b63083af368c0431` | Radix-2 roots/rotations for `QMF_FFT_RADIX2_MATH_V1`; compiled values are not used. | F04 P1.4 validator not implemented. |
| `parameters/descriptor-mathematical.json` | `GENERATED`, 4861 bytes, SHA-256 `a493fbdaf2dab30e940cae9976ed3305f3af083e5614ab0cbd6889c71a02b197` | 128 gain words and 127 threshold words. | Audio acceptance and F04 P1.4 validation pending. |
| Cold constructor arrays and sentinels | `NO_RUNTIME_PARAMETER` | Supplement pins all typed values and the full recursive wire grammar. `None` sentinels stay distinct from raw zero bytes. | Implement only from the typed recipe; compare KAT after project encoding. |

`parameters/spread-authorized-v1.json` is an authorized non-secret FILE provider but belongs to the upstream `spatial_control/F03W` installed48 producer path, not this owner root. `parameters/diffusion-mathematical-v1.json` and `parameters/diffusion-constructor-mathematical-v1.json` are GENERATED upstream providers for `spatial_control/F03W`; they do not place diffusion instances in `F04CompositionState`. The selected production diffusion configuration remains unsupplied for that route.

## Future post-window boundary

The separate `current-features-integration-contract.v1` supplement defines an opaque linear `PostWindowAudioPacket` at `INSTALLED48_POST_WINDOW_48X3072`, carrying finite source-major `F32[48][3072]` plus programme/attempt, transaction/callback, profile/provider lineage, spatial-control before generation and complete-root digest, render receipt, and payload SHA-256. Its supplement manifest SHA-256 is `e240701f0128877baefb305152f10553de9288dc05298452d3605011e63603ad`; `CONTRACT.md` is 3919 bytes / `3cf97a638fbe02dc36edf8a29534493e2b7de8947b858f749eefdef636cae0a5`, and `API-SCHEMA.json` is 786 bytes / `280ebef7034797e7efa84e2e3514cbde05ffd973ac644c5f13e804d057b778c6`.

The future producer stage belongs at the F03W `W3072` / installed48 post-window boundary under `spatial_control`, before F04C consumes the same callback's installed48 data. The supplement defines the packet interface but does not itself name this producer owner; the attribution here follows the approved F03W ownership of installed48 producer histories and must be confirmed in the producer contract before implementation. The render producer is deferred. The linear packet's exact fanout to CurrentFeatures and the F04C consumer is unresolved: no clone or independent lineage reconstruction is authorized.

## Gate result

The GTC owner slot, candidate schema, nine top-level fields, owner split, clocks, receipt boundary, six temporal constructors, and full D-node wire grammar are mapped. P1.3-E1 may now implement cold storage and staging. The overall spatial_audio root remains `MAPPING_COMPLETE_BLOCKED` on F04 provider validation, render/fanout, and parent CallbackTransaction. The KAT is not local reconstruction evidence until generated bytes are compared. No full callback, audio-chain, DSP, or product readiness is claimed. See [SPATIAL-AUDIO-COLD-STAGING-BLOCKER.md](SPATIAL-AUDIO-COLD-STAGING-BLOCKER.md) for the resolved spec blocker and implementation boundary.
