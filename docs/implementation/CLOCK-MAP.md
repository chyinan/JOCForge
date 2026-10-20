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
| F03G | — | — | INVENTORIED |
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
| F06L | — | — | INVENTORIED |
| F06I | — | — | INVENTORIED |
| F06W | — | — | INVENTORIED |
| F06KG | — | — | INVENTORIED |
| F06TP | — | — | INVENTORIED |
| F06ND | — | — | INVENTORIED |
| F06NR | — | — | INVENTORIED |
| F06NI | — | — | INVENTORIED |
| F06NF | — | — | INVENTORIED |
| F06WH | — | — | INVENTORIED |
| F06U | — | — | INVENTORIED |
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
| F13 | — | — | INVENTORIED |
| F13F | — | — | INVENTORIED |
| F13T | — | — | INVENTORIED |
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
| F05O/oamd-rowdata-body-encoder | — | `The encoder is stateless and consumes no callback, source, writer, or AU clock.` |
| F05O/oamd-single-rowdata-body-decoder | — | `The body reader is stateless and takes no callback, source, writer, or AU clock.` |
| F05T/trim-section-codec | — | `The trim codec and selected DBMD projection are stateless and consume no callback, source, writer, AU, or transaction clock.` |
