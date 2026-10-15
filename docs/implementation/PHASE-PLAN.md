# Phase 1 计划

P1.1 仅含两个 architecture/foundation task；P1.2～P1.4 保持 HOLD。

| task_id | functional_scope | covered_clauses | production_dimensions | validation_profile | status |
|---|---|---|---|---|---|
| CLOCK-FOUNDATION | {"F02": "SCOPED_CLAUSES_ONLY", "XSTJ": "SCOPED_CLAUSES_ONLY"} | `callback g`<br>`source s=g-2`<br>`dispatch d=g-1`<br>`writer=d-1`<br>`published AU=s-2`<br>`source0/1 suppression`<br>`sample-count arithmetic` | — | — | HOLD |
| SELECTED-PROFILE-GATE | {} | — | `AuthoredInputProfile`<br>`CoreCarrierProfile`<br>`JocDownmixProfile`<br>`SubstreamTopology`<br>`CodecRateProfile`<br>`EmdfCarriageProfile` | `RenderTarget` | HOLD |

## P1.1 failing-first tests

1. compile-fail cross-domain newtype interchange
2. g=0/1 source underflow rejection
3. s=0/1 suppression
4. s=2 to AU0
5. g-dispatch-writer boundaries
6. sample counts 1,1535,1536,1537,6522906
7. 4247 normal plus 4 drain
8. 4249 sources
9. 4247 published AUs
10. both approved carriage modes accepted
11. each production dimension mutation rejected
12. RenderTarget does not affect production resolution
13. profile rejection creates no staged root or temp output and leaves committed state unchanged

## 后续 slice

- P1.2：HOLD。
- P1.3：HOLD。
- P1.4：HOLD。
