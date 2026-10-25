# 时钟映射

由 IMPLEMENTATION-MAP.json 生成，不得手工修改。

| functional_id | domains | covered_clauses | inventory_status |
|---|---|---|---|
| STRC | — | — | INVENTORIED |
| GTC | — | — | INVENTORIED |
| XSTJ | `AdmSampleCount`<br>`NormalCallbackCount`<br>`FrontendCallbackIndex`<br>`CodecSourceIndex`<br>`JocDispatchIndex`<br>`JocGeneratedWriterIndex`<br>`OrdinaryPayloadWriterIndex`<br>`JocPoppedWriterIndex`<br>`PublishedAuIndex` | `F01/F02 input sample-count clauses`<br>`callback g`<br>`codec source s=g-2`<br>`JOC dispatch d=s+1=g-1`<br>`generated JOC writer=d-1=s`<br>`source0/1 suppression`<br>`source s>=2 to published AU=s-2`<br>`published AU ordinary/JOC writer offsets`<br>`programme count arithmetic` | INVENTORIED |
| F01 | — | — | INVENTORIED |
| F01R | — | — | INVENTORIED |
| F01G | — | — | INVENTORIED |
| F01B | — | — | INVENTORIED |
| F01GL | — | — | INVENTORIED |
| F01C | — | — | INVENTORIED |
| F01E | — | — | INVENTORIED |
| F02 | `AdmSampleCount`<br>`NormalCallbackCount`<br>`FrontendCallbackIndex`<br>`CodecSourceIndex`<br>`JocDispatchIndex`<br>`JocGeneratedWriterIndex`<br>`OrdinaryPayloadWriterIndex`<br>`JocPoppedWriterIndex`<br>`PublishedAuIndex` | `F01/F02 input sample-count clauses`<br>`callback g`<br>`codec source s=g-2`<br>`JOC dispatch d=s+1=g-1`<br>`generated JOC writer=d-1=s`<br>`source0/1 suppression`<br>`source s>=2 to published AU=s-2`<br>`published AU ordinary/JOC writer offsets`<br>`programme count arithmetic` | INVENTORIED |
| F03 | — | — | INVENTORIED |
| F03G | — | `Target-slot installation is a pure per-callback value transition and owns no callback/source/AU clock.` | INVENTORIED |
| F03C | — | — | INVENTORIED |
| F03U | — | — | INVENTORIED |
| F03F | — | — | INVENTORIED |
| F03FC | — | — | INVENTORIED |
| F03R | — | — | INVENTORIED |
| F03P | — | — | INVENTORIED |
| F03N | — | — | INVENTORIED |
| F03S | — | — | INVENTORIED |
| F03FD | — | — | INVENTORIED |
| F03W | — | — | INVENTORIED |
| F04 | — | — | INVENTORIED |
| F04C | — | — | INVENTORIED |
| F04A | — | — | INVENTORIED |
| F04F | — | — | INVENTORIED |
| F04FM | — | — | INVENTORIED |
| F04FCM | — | — | INVENTORIED |
| F04G | — | — | INVENTORIED |
| F04GA | — | — | INVENTORIED |
| F04P | — | — | INVENTORIED |
| F04PG | — | — | INVENTORIED |
| F04R | `FrontendCallbackIndex`<br>`ReceiverCallbackCount`<br>`SpatialDispatchIndex` | `g=0..4246 normal input callbacks; g=4247..4250 cached drain`<br>`For g>=2, source s=g-2 and receiver callback r=s; g=0/1 prime Type10 and skip F04R`<br>`For g>=2, receiver generation r+1=g-1 and JOC dispatch d=g-1=s+1`<br>`Each actual cached-drain callback with Ready Type10 advances the full ModelState once; EOF adds no synthetic F04R callback or flush` | INVENTORIED |
| F04D | `FrontendCallbackIndex`<br>`ReceiverCallbackCount` | `g=0/1 skips descriptor; g>=2 uses r=g-2 once per real receiver callback`<br>`control selection after temporal0..2 and before temporal3`<br>`gain ramp consumes temporal3 after for six lanes`<br>`cached drain is a real callback; no synthetic descriptor flush` | INVENTORIED |
| F04Q | — | — | INVENTORIED |
| F04RB | `FrontendCallbackIndex`<br>`ReceiverCallbackCount`<br>`JocDispatchIndex` | `g<2 is Type10 priming; no receiver or mode0 fitting callback`<br>`for g>=2, r=g-2; F04RB callback_index_zero_based is r and generation is r+1=g-1`<br>`the same frontend callback's dispatch index is d=g-1=r+1; fitting does not execute J05`<br>`each receiver callback consumes 24 ordered 64-sample slots and advances each fitting owner once`<br>`KAT cases are g2/r0/gen1/d1, g3/r1/gen2/d2, and g4/r2/gen3/d3`<br>`cached-drain callbacks with Ready Type10 are real callbacks; no synthetic QMF flush or EOF callback` | INVENTORIED |
| F04QF | — | — | INVENTORIED |
| F04QP | — | — | INVENTORIED |
| F05 | — | — | INVENTORIED |
| F05Q | — | — | INVENTORIED |
| F05QH | — | — | INVENTORIED |
| F05J | — | — | INVENTORIED |
| F05JC | — | — | INVENTORIED |
| F05JL | — | — | INVENTORIED |
| F05JS | — | — | INVENTORIED |
| F05O | — | — | INVENTORIED |
| F05T | — | — | INVENTORIED |
| F05H | — | — | INVENTORIED |
| F05L | — | — | INVENTORIED |
| F05R56 | `FrontendCallbackIndex`<br>`ReaderInvocationIndex`<br>`HistoryProducerGeneration` | `g is zero-based FrontendCallbackIndex; g=0,1 advance marker/history and target snapshot only`<br>`reader begins only for g>=2; q is ReaderInvocationIndex(g-2)`<br>`history producer generation is carried independently per delivered regular row; cold generation is -1`<br>`direct history row provenance is current g; regular rows preserve delivered producer generation`<br>`one successful g>=2 callback proposes one receipt with reader invocation q and all 16 row generations`<br>`callback g is advanced only by the enclosing spatial_control transaction; this stage has no commit` | INVENTORIED |
| F06 | — | — | INVENTORIED |
| F06L | — | `Ordered log, energy metric, and per-row peak metrics are pure scalar/row operations and own no window, callback, source, or AU clock.` | INVENTORIED |
| F06I | — | `This slice owns only its internal energy-pair counter: event at count 20, then every five accepted pairs; it does not own F06W meter/sample scheduling or programme clocks.` | INVENTORIED |
| F06W | — | `Stateless accounting derives normal/drain callback spans for an explicit positive source sample count; it does not own an audio callback or programme clock.`<br>`One energy-window transition consumes an explicitly supplied complete six-lane 1024-sample block, retains its 960-window sample count, and owns no absolute callback or block index.` | INVENTORIED |
| F06KG | — | — | INVENTORIED |
| F06TP | — | — | INVENTORIED |
| F06OF | — | — | INVENTORIED |
| F06NR | — | — | INVENTORIED |
| F06NI | — | — | INVENTORIED |
| F06NF | — | — | INVENTORIED |
| F06WH | — | — | INVENTORIED |
| F06U | — | `One explicit accepted energy pair is one period-one event; F06U owns no callback, frame, window, source, or AU clock.` | INVENTORIED |
| F06DG | — | — | INVENTORIED |
| F06D | `SharedGainDelayCallbackIndex` | `One admitted six-row 1536-sample call advances the leaf callback index exactly once and executes 48 ordered 32-sample blocks.`<br>`Reset reconstructs callback index zero, the exact cold parameter state and all six zero histories.` | INVENTORIED |
| F06M | — | — | INVENTORIED |
| F06OG | — | — | INVENTORIED |
| F06OM | — | — | INVENTORIED |
| F06OP | — | — | INVENTORIED |
| F06OE | — | — | INVENTORIED |
| F06QT | — | — | INVENTORIED |
| F06QM | — | — | INVENTORIED |
| F06QS | — | — | INVENTORIED |
| F06OD | — | — | INVENTORIED |
| F06OS | — | — | INVENTORIED |
| F06OB | — | — | INVENTORIED |
| F06OR | — | — | INVENTORIED |
| F06OC | — | — | INVENTORIED |
| F06ND | — | — | INVENTORIED |
| F07 | — | — | INVENTORIED |
| F07A | — | — | INVENTORIED |
| F08 | — | — | INVENTORIED |
| F08NZ | — | — | INVENTORIED |
| F09 | — | — | INVENTORIED |
| F09E | — | — | INVENTORIED |
| F09PA | — | — | INVENTORIED |
| F09G | — | — | INVENTORIED |
| F09LB | — | — | INVENTORIED |
| F10 | — | — | INVENTORIED |
| F11 | — | — | INVENTORIED |
| F12 | — | — | INVENTORIED |
| F12S | — | — | INVENTORIED |
| F12SC | — | — | INVENTORIED |
| F12A | — | — | INVENTORIED |
| F13 | `CodecSourceIndex`<br>`JocDispatchIndex`<br>`PublishedAuIndex` | `source s advances dispatch s+1; AU is absent for s<2 and s-2 otherwise`<br>`ordinary payload writers join AU j=s-2; ID14 writer joins as s-1`<br>`source 0 and 1 complete queue/timer work without publication` | INVENTORIED |
| F13F | `CodecSourceIndex`<br>`PublishedAuIndex` | `push then immediate pop in ID11/ID1/ID2/ID14 order`<br>`ordinary pop writer s-2`<br>`ID14 pop writer s-1` | INVENTORIED |
| F13T | `CodecSourceIndex` | `one ID1 timer update per 1536-sample source s including priming sources`<br>`six signed remaining counters and cache slots`<br>`counter_low_s16 decrement range, signed-16 wrap, flag clear/set/preserve, and counter-flag refresh suppression`<br>`at most one expired timer refresh per call in priority order`<br>`mode 0 versus mode 1 behavior`<br>`fixed constructor scalars counter_extra_u16=0, group_s16=-1, codec_s16=-1` | INVENTORIED |
| F13C | — | — | INVENTORIED |
| F13D2 | — | — | INVENTORIED |
| F14 | — | — | INVENTORIED |
| F14K | — | — | INVENTORIED |
| F15 | — | — | INVENTORIED |
| F15P | — | — | INVENTORIED |
| OWNJ | — | — | INVENTORIED |

## Direct sub-slice clocks

由 IMPLEMENTATION-MAP.json 生成，不得手工修改。

| slice_id | domains | covered_clauses |
|---|---|---|
| F01R/selected-riff-pcm24-reader-and-hop | — | `Parsing one immutable complete byte slice and one explicit sample hop owns no callback, sample-progress, source, or AU clock.` |
| F01G/selected-real-axml-chna-host-identity-and-source-join | `CodecSourceIndex` | `37 regular physical hosts, ascending excluding LFE host3, map to CodecSourceIndex 11..47; this is identity projection only, not an audio clock.` |
| F01B/candidate-normalized-bed-source-points | — | `Bed role point generation is stateless and owns no source, callback, or publication clock.` |
| F01GL/candidate-virtual-prefix-q15-geometry | — | `Prefix Q15→F32 geometry is stateless; source slots are value indices only, not a callback or audio clock.` |
| F01E/selected-job-xml-profile-normalization | — | `Source sample count is an input field only. The parser owns no reader, callback, source, dispatch, or AU clock and does not convert file positions to source samples.` |
| F01E/selected-job-resource-and-adm-event-admission | `AdmSampleCount` | `the validated source extent admits zero-start single-event metadata before the graph projection is held`<br>`this value boundary owns no callback/source/dispatch/AU advancement` |
| F01E/selected-dbmd-authored-mode-and-track-field-producer | — | `DBMD section parsing and track-field projection are stateless; physical host association must come from the separate selected F01G graph receipt.` |
| F01E/selected-object-event-control52-row-production | — | `Source sample count and event duration are validation inputs only. This ordered row-set builder owns no callback, source, dispatch, AU, hold, or reset clock.` |
| F01E/primary-source-bound-control38-value-production | `AdmSampleCount` | `The source sample count validates event extent only; this pure producer owns no callback, persistent state, commit, reset, drain, or publication clock.` |
| F03G/target-slot-install | — | `Pure per-callback value transition; no clock or committed state is owned.` |
| F03C/candidate-selector | — | `Each selector call initializes cold scratch; no callback/source/AU counter or cross-call selector state is owned.` |
| F03C/selector-score-choice | — | `No state or clock is owned.` |
| F03C/selector-pairwise-affinity | — | `No state or clock is owned.` |
| F03U/u04-membership-projection | — | `Projection is a stateless local value transform; it does not own callback/source/AU clocks.` |
| F03U/u04-refinement-admission-gate | — | `The gate has no state or clock ownership.` |
| F03R/selected-profile-feature-control-and-value-only-source-flags | `FrontendCallbackIndex`<br>`MeterCallbackIndex`<br>`AdmSampleIndex`<br>`PhysicalHostIndex`<br>`CodecSourceIndex` | `selected feature controls carry the selected resolved profile, explicit callback index, and physical host`<br>`host10 authored Control52 word10=1 and typed F03R feature diffuse=0 coexist for callback0`<br>`the value-only SourceFlags transition advances callback history from the prior raw host rows and returns no packet or parent commit` |
| F03N/rectangular-nnls-solver | — | `Every solve starts cold and owns no callback, source, AU, or cross-call NNLS state.` |
| F03S/outer-coefficient-postprocess | — | `The postprocess is stateless and has no source, callback, AU, or parent transaction clock.` |
| F03S/outer-energy-fallback | — | `The function is stateless and owns no source, callback, AU, or parent transaction clock.` |
| F03S/outer-geometric-correction | — | `The geometry primitive is stateless and owns no source, callback, AU, or parent transaction clock.` |
| F03S/outer-active-update | — | `The active-update primitive is stateless and owns no source, callback, AU, or parent transaction clock.` |
| F03S/outer-empty-matrix-skip | — | `The empty matrix skip owns no source, callback, AU, or parent transaction clock.` |
| F03W/selected-primary-installed48-source-map-and-activity | `FrontendCallbackIndex`<br>`AdmSampleIndex`<br>`CodecSourceIndex`<br>`PhysicalHostIndex` | `Normal input callbacks g=0..4246; cached drain callbacks g=4247..4250`<br>`source s=g-2 only for g>=2; source0/1 fully advance history but suppress publication`<br>`source s>=2 publishes AU-independent installed48 state for source s; no AU or dispatch clock is consumed by this slice`<br>`reader position is min(1536*g, 6,522,906), with the final 486 samples padded positive-zero`<br>`each cached drain reuses the cached padded frame/control and advances callback-local activity history once` |
| F04/independent-meter-installed48-postwindow-leaf | `MeterCallbackIndex`<br>`AdmSampleIndex`<br>`FrontendCallbackIndex`<br>`CodecSourceIndex`<br>`PhysicalHostIndex` | `the selected NORMAL source interval begins at 1536*g; output generation is g+1`<br>`the leaf has fresh previous PCM, previous-current rows and per-host diffusion state; no main root alias or commit`<br>`corrected actual F04A generations g0/g1/g2 are recorded; no independent expected comparison, F03F stage, or callback root was committed` |
| F04C/whole-spatial-audio-j05-composer | `FrontendCallbackIndex`<br>`CodecSourceIndex`<br>`ReceiverCallbackCount`<br>`JocDispatchIndex`<br>`ReaderInvocationIndex`<br>`HistoryProducerGeneration` | `g0/g1 priming`<br>`g>=2 s=r=g-2`<br>`d=g-1`<br>`full parent g to g+1`<br>`reader envelope g/invocation g-2`<br>`component g and LFE content g-2` |
| F04A/independent-meter-installed48-postwindow-leaf | `MeterCallbackIndex`<br>`AdmSampleIndex`<br>`FrontendCallbackIndex`<br>`CodecSourceIndex`<br>`PhysicalHostIndex` | `corrected actual output used g0/g1/g2 NORMAL source intervals beginning at 0/1536/3072 and generations 1/2/3`<br>`the leaf has fresh previous PCM, previous-current rows and per-host diffusion state; no main root alias or commit`<br>`actual F04A values are noncommitting and have no independent expected-hash comparison; no F03F stage or callback root is committed` |
| F04F/single-tail-diffusion-filter-block-preview | — | `The leaf consumes one caller-supplied F32 block; sample order is increasing within that block.`<br>`It owns no callback, source, dispatch, writer, AU, or transaction clock.`<br>`F04F preview returns a next-state proposal only; callback commitment remains the enclosing spatial_control/spatial_audio programme owner authority.` |
| F04F/single-tail-active-inactive-callback-lifecycle | — | `The lifecycle preview consumes at most one caller-supplied previous/current frame pair and owns no callback/source/AU clock.`<br>`Inactive removes the one tail; active with no before-state constructs a cold instance; active with a complete before-state advances it once.`<br>`The four previous outputs are read only from that same tail's cache; filter next-state commit remains caller-owned.` |
| F04G/selected-p11-prefix-membership-producer | — | `The membership producer is stateless; it consumes no callback, source, writer, AU, or transaction clock.` |
| F04RB/actual-receiver-to-joc-qmf-projection | `FrontendCallbackIndex`<br>`CodecSourceIndex`<br>`ReceiverCallbackCount`<br>`JocDispatchIndex` | `g>=2 maps to source and receiver s=r=g-2`<br>`dispatch d=g-1=s+1 and the receiver/fitting generation after is s+1`<br>`the projection consumes one receiver proposal and cannot advance receiver or descriptor state again` |
| F05/selected-coefficient-fit-retry-and-payload | `parent-owned JocDispatchIndex` | `one successful temporal transition per admitted J05 dispatch`<br>`the detector consumes current24 slots only; previous24 remains a later weighted-fit input`<br>`a representation retry reuses the same temporal proposal and cannot advance it again` |
| F05Q/quantized-huffman-body-writer | — | `The body writer is stateless across programme callbacks and emits one local proposal from the supplied point sequence; no programme/source/AU clock or parent commit is owned here.` |
| F05Q/id14-header-body-writer | — | `The selected ID14 writer is a stateless bitstream stage; sequence_count is a wire field, not an owned programme clock.` |
| F05QH/huffman-codebook-table-provider | — | `Codebook validation and table-level encoding/decoding have no callback/source/AU clock and perform no parent commit.` |
| F05JL/synthetic-coordinator-receipt-validation | `synthetic dispatch`<br>`derived callback/source/writer/AU values` | `source=dispatch-1`<br>`writer=dispatch-1`<br>`callback zero-based=dispatch+1`<br>`callback one-based=dispatch+2`<br>`AU null for dispatch<3 else dispatch-3` |
| F05JL/selected-fit-dispatch-and-id14-proposal | `JocDispatchIndex`<br>`CodecSourceIndex`<br>`sequence_counter` | `dispatch d starts at 1`<br>`writer/source=d-1`<br>`wire sequence 0,0,1..1023,1`<br>`failed trials do not advance staged clocks` |
| F05O/oamd-rowdata-body-encoder | — | `The encoder is stateless and consumes no callback, source, writer, or AU clock.` |
| F05O/oamd-single-rowdata-body-decoder | — | `The body reader is stateless and takes no callback, source, writer, or AU clock.` |
| F05O/single-frame-oamd-body-assembler | — | `The single-frame body assembler is stateless and consumes no callback, source, writer, or AU clock.` |
| F05T/trim-section-codec | — | `The trim codec and selected DBMD projection are stateless and consume no callback, source, writer, AU, or transaction clock.` |
| F06/programme-measurement-finalize-projection | `meter callback count`<br>`energy window count` | `selected 4247 normal callbacks`<br>`three zero drains`<br>`6794 complete energy windows`<br>`finalize produces no new callback` |
| F06/post-render-meter-callback-composition | `ObjectRenderCallbackIndex`<br>`ObjectRenderCallbackGeneration`<br>`SharedGainDelayCallbackIndex`<br>`MeterCallbackIndex`<br>`MeterUpstreamGeneration`<br>`MeterSamplePosition`<br>`MeterBlockIndex`<br>`EnergyRecordCount` | `One meter callback index binds the direct-LFE ring, OAR generation g+1, DV32, and sample queue.`<br>`The queue retains actual six-lane sample words and emits only complete 1024-sample blocks.`<br>`Each complete block advances true peak and K/window once; each complete 960-sample record advances integrated and ungated state once.`<br>`The combined result carries LFE and post-render after values as proposals without EOF flush or local installation.` |
| F06/independent-meter-prepass-callback-commit | `MeterSourceSampleCount`<br>`MeterCallbackIndex`<br>`ObjectRenderCallbackGeneration`<br>`MeterBlockIndex`<br>`EnergyRecordCount` | `Existing meter leaf clocks require ceil(N/1536) normal callbacks followed by three zero-input drains.`<br>`LFE callback index, upstream generation, renderer callback and sample queue callback remain joined by their typed owner checks.`<br>`A staged callback is a one-shot mutable borrow; both owner after states install in one root replacement.`<br>`A stage failure changes only prepass lifecycle to FAILED; full reset reconstructs both local owners from cold.` |
| F06L/ordered-log-energy-peak-accumulator-window-and-reduction | — | `Pure scalar and one-row operations; no meter window or programme clock is owned.` |
| F06I/integrated-meter-synthetic-state-and-event-transition | — | `Only the state-local energy-pair counter and period=5 group/event transitions are implemented; no meter sample/window scheduler is included.` |
| F06W/meter-output-sample-accounting-and-span-ledger | — | `The computed callback_index is local to a stateless ledger call; no live sample, audio callback, or programme clock is owned.` |
| F06W/k-weighted-complete-block-and-energy-window | — | `Each call consumes exactly one caller-supplied complete 1024-sample block per lane.`<br>`The state retains only the 0..959 sample position within the current energy window; it owns no absolute callback or block index.` |
| F06KG/selected-k-matrix-provider-generator | — | `The provider/generator is immutable and owns no window, callback, source, AU, or per-block meter clock.` |
| F06TP/selected-six-channel-true-peak-block-cascade | — | `Each call consumes one explicit 1024-sample six-channel source block; block/callback/source/AU scheduling and EOF accounting remain external. FIR state advances only through the returned next state.` |
| F06OF/meter-lfe-delay-and-oar-packet-assembly | `MeterCallbackIndex`<br>`ObjectRenderCallbackGeneration`<br>`MeterLfeCallbackOrigin` | `Cold ring write/read slots are 0/1 and all three frame and origin slots are absent.`<br>`One input frame and callback index advance the ring once; output g uses input generation g-2 after the first two callbacks.`<br>`The OAR packet generation must equal meter callback index g+1 and packet regular PCM/control rows come from the same input batch.`<br>`The three meter drain callbacks require zero raw direct-LFE input; the ring output still comes from its retained frame slot.` |
| F06OF/meter-in-memory-pcm24-source-cursor | `MeterSourceSampleCount`<br>`MeterSourcePosition`<br>`MeterCallbackIndex`<br>`MeterUpstreamGeneration`<br>`MeterCallbackCount` | `Normal callback count is ceil(N/1536); each normal frame reads min(1536,N-cursor) then pads the raw input rows to 1536.`<br>`The source cursor advances only for normal host samples and reaches N at end of source.`<br>`Exactly three callbacks after the normal count are zero-input drains; they do not read source data or advance the cursor.`<br>`Every frame generation is callback index g+1; the frame carries its before/after source cursor and host/padding counts.` |
| F06NR/single-lane-secondary-overlap-carry | — | `One overlap block is a stateless six-lane-independent transition over an explicit old carry.`<br>`The next carry is a bitwise copy of transformed words 64..192; it does not reuse overlap output.` |
| F06NR/selected-secondary-carrier-routing | — | `Carrier assembly is stateless over an explicit 5x6x256 input.`<br>`Each output carrier row is fully initialized; lane block 0 is omitted and the LFE row remains positive zero.` |
| F06NI/selected-secondary-inverse256-wrapper | — | `One inverse256 call owns no source-frame, block, callback, or programme clock.`<br>`Long and short paths use independent descriptor scratch and write all 256 output words once.` |
| F06NF/selected-secondary-fft-network | — | `One FFT invocation is stateless and owns no callback, source, AU, or programme clock.`<br>`FFT64 and FFT128 accept exactly 2N interleaved finite F32 words and write every output word once.` |
| F06U/ungated-single-slot-mean-and-di-off-getter | — | `Each explicitly supplied, admitted energy pair causes one period-one event; no callback/window/source/AU clock is owned.` |
| F06D/selected-shared-gain-delay-state-and-callback-transition | `SharedGainDelayCallbackIndex` | `Exactly one leaf callback index increment after all 48 blocks succeed; failure leaves the prior state unchanged.` |
| F06ND/selected-none-drc-rf-line-scalar-quantizer | — | `A scalar quantizer invocation is stateless and owns no source, AU, callback or programme clock.` |
| F06ND/selected-none-drc-scalar-normalizer | — | `A scalar normalization call is stateless and owns no source, AU, callback or programme clock.` |
| F06ND/selected-none-drc-dialnorm-offset | — | `A dialnorm offset mapping is stateless and owns no source, AU, callback or programme clock.` |
| F07/input-history-block-transition | — | `The helper advances exactly one caller-ordered 256-sample block and owns no absolute source-frame clock.` |
| F07/structure-refresh-flag-projection | — | `One stateless six-block projection; no source, frame, or AU counter is retained.` |
| F07/selected-dither-flag-plan | — | `Consumes one explicitly ordered six-block frame and returns the next previous_short projection; no absolute clock is owned.` |
| F07/actual-d05-transient-channel-block | — | `One input block advances one caller-owned lane state; 36-call ordering is test input, not a programme source or AU clock owner.` |
| F07A/selected-d05-transient-parameter-provider | — | `Stateless identity and typed-parameter loading; no frame or source clock is owned.` |
| F09/exponent-refresh-state-transition | — | `Consumes one caller-provided [begin, end) row window and owns no callback, source, AU, or lifecycle clock.` |
| F09/raw-exponent-from-power-bits | — | `Consumes one caller-supplied U32 word, bias, and cap; owns no state or clock.` |
| F09/coefficient-pair-to-power-row | — | `Consumes caller-owned coefficient rows synchronously; owns no callback, source, dispatch, AU, or persistent state clock.` |
| F09/lfe-coefficient-pair-exponent | — | `Consumes caller-owned two-block coefficient rows synchronously; owns no callback, source, dispatch, AU, or persistent lifecycle clock.` |
| F09/long-transform-refresh-composition | — | `Each helper call consumes one synthetic block and returns next caller-owned branch history; consecutive test calls do not identify ADM sample positions or establish the product callback/source/AU clock.` |
| F09/lfe-long-transform-to-raw | — | `Two branch history values are caller-owned inputs/outputs retained across the two synthetic test calls only; product analysis generation, source/AU mapping, and reset/clock ownership are not assigned here.` |
| F09/transform-history-owner-and-workspace-initializer | — | `Controlled same-owner history carry is tested across two calls; no absolute sample, analysis generation, callback, AU, or EOF clock is inferred.` |
| F09/selected-configuration-record-import | — | `One synchronous source-record import only; no source-frame, callback, AU, or persistent owner clock is created.` |
| F09E/refresh-difference-index-map | — | `The function is pure, stateless, and owns no callback, source, AU, or commit clock.` |
| F09PA/selected-policy-parameter-provider | — | `Stateless provider admission; no numeric, db, gain, transform, callback, source, or AU clock advances.` |
| F09G/precomputed-means-to-db-selector | — | `Pure one-block selector calculation; returns the next U16 value without committing cross-block or frame state.` |
| F09G/four-lane-mean-square-256 | — | `One stateless 256-sample call; no source, block progression, callback, or AU state is retained.` |
| F09G/gain-history-peak-state-pass0-gate1-format7 | — | `The helper processes one caller-supplied block; caller metadata orders block/frame calls. Previous peak and Q15 state carry across supplied calls, while summary/gain rows are overwritten by block index. No absolute ADM/sample/AU clock is assigned.` |
| F09LB/selected-lfe-gate-and-mantissa-boundary | — | `The selected coefficient stage owns no source/AU counter; each explicit stage/finish pair describes one local candidate transaction.` |
| F10/fullband-exponent-row-normalization | — | `Stateless one-row calculation over caller-ordered bins; no source-frame clock or committed state is owned.` |
| F10/exponent-difference-triple-packing | — | `Stateless packing of one caller-ordered group; no source-frame or AU clock is owned.` |
| F12/grouped-quantizer-helper | — | `Consumes an explicitly ordered item slice; owns no callback, source, AU, or retry clock.` |
| F12/bounded-msb-frame-bit-writer | `FrameBitCursor` | `cursor advances by exactly each admitted field width`<br>`failed width/value/capacity checks do not advance the cursor` |
| F12S/mantissa-quantizer | — | `The quantizer is stateless and has no callback, source, AU, or trial-search clock ownership.` |
| F13/selected-metadata-owner-source-stage | `CodecSourceIndex`<br>`JocDispatchIndex`<br>`PublishedAuIndex` | `source s`<br>`dispatch s+1`<br>`AU absent for s<2 else s-2`<br>`ordinary writer s-2`<br>`ID14 writer s-1` |
| F13C/selected-bsi-count-and-payload-import | — | `The scalar formula, count admission projection, producer and literal importer copy are stateless calls; no source, dispatch, AU, or programme clock is owned.` |
| F13C/selected-emdf-carrier-planner-reservation-writer-and-publication-gate | `FrontendCallOrdinal`<br>`PublishedAuOrdinal`<br>`CarrierBitCursor` | `Frontend calls are one-based; published AU ordinals are zero-based and advance only for nonzero valid output bytes.`<br>`The six-call warmup gate emits zero bytes for calls 1..2 and 4096 bytes for calls 3..6; its progress counter saturates at 4608 and is not an ADM/source-sample clock.`<br>`Natural published AU 0..3 join frontend/parent calls 3..6 and JOC producer calls 2..5. Absolute ADM sample and host-frame binding are not available.`<br>`Reservation and payload positions use CarrierBitCursor, distinct from frontend and AU ordinals.` |
| F13C/selected-input-eof-lifecycle-observation-replay | `ExternalControlPosition`<br>`RelativeReaderCursorSamples`<br>`SelectedItemSampleCoordinate`<br>`RendererSpanSamples`<br>`RendererProgressSamples`<br>`FrontendCallOrdinal`<br>`PublishedAuOrdinal` | `Four external process calls each carry 38 source-index/control-position records at API positions 0, 1536, 3072, 4608; these are not absolute ADM/WAV offsets.`<br>`For one selected item (source sample coordinates 2880000..2886000), cursor-before 0/1536/3072/4608 matches the relative API position; selected source sample coordinates are derived as 2880000/2881536/2883072/2884608.`<br>`The final read_float call requests/returns 1392 samples while its returned packet reports 1536; PCM and padding contents are not observed.`<br>`Selected constructor and CompositeMaster descriptor match by selection index and exact fields; pointer alias is not proved. Absolute file byte offset and host clock remain unbound.`<br>`Frontend source calls 1 and 2 originate inside external process calls 3 and 4; frontend source calls 3 through 6 originate inside active flush calls 1 through 4.`<br>`Four active flushes produce published AU ordinals 0..3; the fifth empty flush does not advance renderer/frontend/planner state.`<br>`Renderer span values and renderer/backend warmup progress are distinct from API control positions and from absolute source/file clocks.` |
| F13C/selected-reader-partial-frame-input-padding | `SelectedReaderActualSampleCount`<br>`SelectedReaderFrameSampleCount`<br>`SelectedReaderTailSampleCount` | `Selected reader frame has 1536 samples per channel and final actual read has 1392; the tail is 144 samples per channel.`<br>`The row table is initialized before the opaque prefix is copied. Tail shape is 38x144 words = 5472 words = 21888 bytes.`<br>`This is an input-provider extent rule only. It does not define a clock, PCM value, file position, or renderer/DSP output tail.` |
| F13C/selected-id11-parser-boundary-evidence | — | `The evidence consistency test owns no source, dispatch, AU, writer, or programme clock.`<br>`The observed parser cursor is recorded as input evidence only; no runtime cursor API is implemented.` |
| F13C/selected-configuration-writer-and-descriptor-bit-budget | — | `Stateless configuration and bit-budget calls own no source, dispatch, AU, or programme clock.` |
| F13C/id14-model-parser-and-native-bsi-source-clarification | — | `This evidence-custody slice owns no runtime source, dispatch, AU, or programme clock.` |
| F13C/selected-id14-semantic-reader-path-applicability | — | `This call-path applicability audit owns no runtime source, dispatch, AU, or programme clock. The four prior serializer rows retain their original call indices as rehashed evidence only.` |
| F13C/container-fragmentation-scratch-parent-observation | — | `Scratch parent observations do not bind to the selected production callback, source, or AU clock.` |
| F13C/selected-production-cold-root-observation | — | `The six frontend/parent calls and 1536-sample gate advances are observed invocation relationships, not absolute ADM position or a new native generation clock.` |
| F13C/id11-native-queue-capacity-and-public-policy-boundary | — | `The queue admission cases own no source, dispatch, AU, or programme clock.` |
| F13C/selected-type11-natural-output-length-bound | — | `The natural-output receipt binds 4249 model outputs to frontend callback indices 2..4250 and 69 secondary-native outputs to source invocations 3..71; it does not define a new product clock.` |
| F13C/controlled-id14-coefficient-to-payload-writer | — | `The controlled coefficient/body writer is stateless per invocation and owns no source, dispatch, AU or programme clock.` |
| F13D2/id2-profile-header-and-body-wire | — | `The profile mapper, local Type11 prefix reader, and ID2 body serializer own no source, dispatch, AU, or persistent clock.` |
| F14K/public-keyless-integrity-owner | `CodecSourceIndex`<br>`JocDispatchIndex`<br>`PublishedAuIndex` | `source_index is the next committed CodecSourceIndex`<br>`dispatch = source_index + 1`<br>`au_index is null for source 0 and 1, then source_index - 2`<br>`the first two source calls execute all three integrity stages with published=false`<br>`source indices >=2 produce published=true and increment published_au_count once`<br>`EOF requires the exact expected source count and publishes max(0, expected_source_count - 2) AUs` |
