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
| F06W | — | `Stateless accounting derives normal/drain callback spans for an explicit positive source sample count; it does not own an audio callback or programme clock.` | INVENTORIED |
| F06KG | — | — | INVENTORIED |
| F06TP | — | — | INVENTORIED |
| F06ND | — | — | INVENTORIED |
| F06NR | — | — | INVENTORIED |
| F06NI | — | — | INVENTORIED |
| F06NF | — | — | INVENTORIED |
| F06WH | — | — | INVENTORIED |
| F06U | — | `One explicit accepted energy pair is one period-one event; F06U owns no callback, frame, window, source, or AU clock.` | INVENTORIED |
| F06DG | — | — | INVENTORIED |
| F06D | — | — | INVENTORIED |
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
| F06OF | — | — | INVENTORIED |
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
| F13T | `CodecSourceIndex` | `one ID1 timer update per 1536-sample source s including priming sources`<br>`six timer periods and input/output field mappings` | INVENTORIED |
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
| F03G/target-slot-install | — | `Pure per-callback value transition; no clock or committed state is owned.` |
| F03C/candidate-selector | — | `Each selector call initializes cold scratch; no callback/source/AU counter or cross-call selector state is owned.` |
| F03C/selector-score-choice | — | `No state or clock is owned.` |
| F03C/selector-pairwise-affinity | — | `No state or clock is owned.` |
| F03U/u04-membership-projection | — | `Projection is a stateless local value transform; it does not own callback/source/AU clocks.` |
| F03U/u04-refinement-admission-gate | — | `The gate has no state or clock ownership.` |
| F03N/rectangular-nnls-solver | — | `Every solve starts cold and owns no callback, source, AU, or cross-call NNLS state.` |
| F03S/outer-coefficient-postprocess | — | `The postprocess is stateless and has no source, callback, AU, or parent transaction clock.` |
| F03S/outer-energy-fallback | — | `The function is stateless and owns no source, callback, AU, or parent transaction clock.` |
| F03S/outer-geometric-correction | — | `The geometry primitive is stateless and owns no source, callback, AU, or parent transaction clock.` |
| F03S/outer-active-update | — | `The active-update primitive is stateless and owns no source, callback, AU, or parent transaction clock.` |
| F03S/outer-empty-matrix-skip | — | `The empty matrix skip owns no source, callback, AU, or parent transaction clock.` |
| F05Q/quantized-huffman-body-writer | — | `The body writer is stateless across programme callbacks and emits one local proposal from the supplied point sequence; no programme/source/AU clock or parent commit is owned here.` |
| F05Q/id14-header-body-writer | — | `The selected ID14 writer is a stateless bitstream stage; sequence_count is a wire field, not an owned programme clock.` |
| F05QH/huffman-codebook-table-provider | — | `Codebook validation and table-level encoding/decoding have no callback/source/AU clock and perform no parent commit.` |
| F05JL/synthetic-coordinator-receipt-validation | `synthetic dispatch`<br>`derived callback/source/writer/AU values` | `source=dispatch-1`<br>`writer=dispatch-1`<br>`callback zero-based=dispatch+1`<br>`callback one-based=dispatch+2`<br>`AU null for dispatch<3 else dispatch-3` |
| F05O/oamd-rowdata-body-encoder | — | `The encoder is stateless and consumes no callback, source, writer, or AU clock.` |
| F05O/oamd-single-rowdata-body-decoder | — | `The body reader is stateless and takes no callback, source, writer, or AU clock.` |
| F05O/single-frame-oamd-body-assembler | — | `The single-frame body assembler is stateless and consumes no callback, source, writer, or AU clock.` |
| F05T/trim-section-codec | — | `The trim codec and selected DBMD projection are stateless and consume no callback, source, writer, AU, or transaction clock.` |
| F06L/ordered-log-energy-peak-accumulator-window-and-reduction | — | `Pure scalar and one-row operations; no meter window or programme clock is owned.` |
| F06I/integrated-meter-synthetic-state-and-event-transition | — | `Only the state-local energy-pair counter and period=5 group/event transitions are implemented; no meter sample/window scheduler is included.` |
| F06W/meter-output-sample-accounting-and-span-ledger | — | `The computed callback_index is local to a stateless ledger call; no live sample, audio callback, or programme clock is owned.` |
| F06KG/selected-k-matrix-provider-generator | — | `The provider/generator is immutable and owns no window, callback, source, AU, or per-block meter clock.` |
| F06TP/selected-six-channel-true-peak-block-cascade | — | `Each call consumes one explicit 1024-sample six-channel source block; block/callback/source/AU scheduling and EOF accounting remain external. FIR state advances only through the returned next state.` |
| F06U/ungated-single-slot-mean-and-di-off-getter | — | `Each explicitly supplied, admitted energy pair causes one period-one event; no callback/window/source/AU clock is owned.` |
| F09LB/selected-lfe-gate-and-mantissa-boundary | — | `The selected coefficient stage owns no source/AU counter; each explicit stage/finish pair describes one local candidate transaction.` |
| F12/grouped-quantizer-helper | — | `Consumes an explicitly ordered item slice; owns no callback, source, AU, or retry clock.` |
| F12S/mantissa-quantizer | — | `The quantizer is stateless and has no callback, source, AU, or trial-search clock ownership.` |
| F13/selected-metadata-owner-source-stage | `CodecSourceIndex`<br>`JocDispatchIndex`<br>`PublishedAuIndex` | `source s`<br>`dispatch s+1`<br>`AU absent for s<2 else s-2`<br>`ordinary writer s-2`<br>`ID14 writer s-1` |
| F13D2/id2-profile-header-and-body-wire | — | `The profile mapper, local Type11 prefix reader, and ID2 body serializer own no source, dispatch, AU, or persistent clock.` |
| F14K/public-keyless-integrity-owner | `CodecSourceIndex`<br>`JocDispatchIndex`<br>`PublishedAuIndex` | `source_index is the next committed CodecSourceIndex`<br>`dispatch = source_index + 1`<br>`au_index is null for source 0 and 1, then source_index - 2`<br>`the first two source calls execute all three integrity stages with published=false`<br>`source indices >=2 produce published=true and increment published_au_count once`<br>`EOF requires the exact expected source count and publishes max(0, expected_source_count - 2) AUs` |
