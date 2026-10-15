# Phase 1 plan

Generated from IMPLEMENTATION-MAP.json. Do not edit by hand.

| task_id | functional_scope | covered_clauses | planned_files | planned_tests | status |
|---|---|---|---|---|---|
| CLOCK-FOUNDATION | {"F02": "SCOPED_CLAUSES_ONLY", "XSTJ": "SCOPED_CLAUSES_ONLY"} | `F02.1: each callback presents a complete 1536-sample host frame; EOF padding is not a codec flush`<br>`F02.2: N>0; A=ceil(N/1536); P=1536*A-N`<br>`F02.2: normal callbacks g=0..A-1; four cached drains g=A..A+3`<br>`F02.2/F02.3: source s=g-2 exists only when g>=2`<br>`XSTJ CLOCK: dispatch d=s+1=g-1; generated JOC writer d-1=s`<br>`XSTJ J03: source 0 and 1 complete queue/source processing but suppress AU publication`<br>`XSTJ J03: source 1 pops JOC writer 0 as CONSUMED_SUPPRESSED`<br>`XSTJ J03: source s>=2 publishes AU j=s-2`<br>`XSTJ CLOCK: publication consumes ordinary writer j and popped JOC writer s-1`<br>`F02.2: A+4 callbacks produce A+2 source frames and A published AUs`<br>`F02.3: EOF adds four cached drains and no codec flush`<br>`F02.4: N=6522906 yields 4247 normal callbacks, 4 drains, 4249 sources, 4247 AUs`<br>`F02.4: for N=6522906, normal callback g=0..4246 and cached drain g=4247..4250`<br>`XSTJ CLOCK: for N=6522906, source s=0..4248; dispatch d=1..4249; generated JOC writer s=0..4248`<br>`XSTJ CLOCK: for N=6522906, published AU j=0..4246; ordinary writer j=0..4246; popped JOC writer s-1=1..4247`<br>`terminal pending writers remain ordinary 4247/4248 and JOC writer 4248` | `crates/jocforge-domain/src/timeline/index.rs`<br>`crates/jocforge-domain/src/timeline/clock.rs`<br>`crates/jocforge-domain/src/timeline/mod.rs` | `crates/jocforge-domain/tests/timeline_clock_contract.rs` | GO |
| SELECTED-PROFILE-GATE | {} | — | — | — | HOLD |

## Gate order

The documentation gate must pass before CLOCK-FOUNDATION implementation work begins. CLOCK-FOUNDATION is the only GO task in this slice.

P1.2, P1.3, P1.4, and SELECTED-PROFILE-GATE remain HOLD.

## Failing-first tests

1. N=0 rejects before state/output creation
2. N=1 produces (1,1535,5,3,1,1)
3. N=1535 produces (1,1,5,3,1,1535)
4. N=1536 produces (1,0,5,3,1,1536)
5. N=1537 produces (2,1535,6,4,2,1)
6. N=6522906 produces (4247,486,4251,4249,4247,1050)
7. g=0/1 has no source; g=2 maps to s=0
8. s=0/1 are suppressed; source 1 pops JOC writer 0 as CONSUMED_SUPPRESSED
9. s=2 publishes AU 0 and consumes ordinary writer 0 plus JOC writer 1
10. s=4248 publishes AU 4246 and consumes ordinary writer 4246 plus JOC writer 4247
11. terminal ordinary writers 4247/4248 and JOC writer 4248 remain pending
12. checked clock conversions reject underflow and overflow
13. distinct callback/source/dispatch/generated/ordinary/popped/AU types fail compile when interchanged
14. XSTJ J03 source0/source1/source2 integration cases match their selected input and expected fields
