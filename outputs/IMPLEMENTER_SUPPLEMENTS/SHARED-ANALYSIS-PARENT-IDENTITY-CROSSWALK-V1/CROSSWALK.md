# Parent-owned shared analysis identity crosswalk

## Decision

The approved contracts already define the source, generation, owner, consumer, and single-commit semantics for F07 through F12. No native `SharedAnalysisReceipt`, native state hash, or DLL field is required.

The reported Rust stop is a missing local constructor and adapter: staged numeric arrays exist, but they are not yet wrapped in a parent-issued type that binds them to the active programme attempt, source, analysis owner, and generation. This is local orchestration work. It is not a missing F07/F09/F10 algorithm or parameter.

## Existing contract

`SourceTransaction(s,d)` is a nested stage of `CallbackTransaction(g)`. Before numeric work, the parent validates `s=g-2`, `d=s+1`, all input generations, and owner tokens (`specs/00-global-transaction-contract.md:118-123`).

The parent then performs the following sequence:

1. Clone the committed `analysis` root into the source stage.
2. Run `SharedAnalysisPrepare` exactly once on that clone. It advances F07 input history/transient once and returns one immutable shared result containing same-source installed/delayed audio, 30 short flags, the primary unquantized spectrum, analysis before/after identity, and `advancement_count=1` (`specs/00-global-transaction-contract.md:123`).
3. F06 borrows that same result for installed audio, primary spectrum, and short flags while mutating only its independent DRC clone. It cannot invoke F07 or commit the analysis clone (`:124`).
4. F08/F09/F10/F11 borrow the same result and continue the six-block numeric plan on the same analysis clone. They produce one `NumericPlanReceipt`; F12 consumes that plan and cannot invoke the whole composer that advances F07 again (`:125,130`).
5. Any consumer failure discards the analysis clone and every other stage root. On complete success, the parent CallbackTransaction swaps all roots once (`:145-153`).

The source-frame contract independently fixes the same behavior: steps F07→F12 execute in order, F08 and F09/F10 operate once, all q candidates read a fixed plan, failed candidates discard scratch only, and one successful best frame commits the source next state once (`specs/12-selected-source-frame-composition.md:13-28,34-54`). Its F12SC adapter explicitly requires source/generation/provider identity agreement and `advancement_count=1` (`:97-99`).

## Required semantic fields

The approved contract requires these fields; they are not optional implementation inventions.

### Shared analysis result

- active source index;
- installed and delayed audio domain identity and immutable data/reference;
- analysis owner before and staged-after generation/identity;
- F07 `advancement_count=1`;
- 30 short flags;
- primary unquantized spectrum identity/data;
- selected provider-set identity needed by downstream checks.

### Numeric plan result

- identity/reference to the exact shared analysis result;
- the same source and analysis generation;
- six-block completion marker;
- short/other flags;
- primary, companion, and LFE spectrum identities/data;
- POWER snapshot and PRIMARY exponent plan;
- gain/db plan;
- normalization/expanded exponent plan;
- converter result;
- provider identities.

These fields come from `specs/00-global-transaction-contract.md:133-137`, `specs/12-selected-source-frame-composition.md:34-46,97-99`, and `parameters/f07-f12-source-state-schema-v1.json:14-29`.

## Minimal local parent-owned wrapper

The contracts do not prescribe a Rust ABI or public constructor. A minimal LOCAL choice is:

```text
SourceStageKey {
    attempt,
    transaction_id,
    source,
    analysis_owner_id,
    analysis_before_generation,
}

SharedAnalysisReceipt { private inner: Arc<SharedAnalysisInner> }
NumericPlanReceipt    { private shared: SharedAnalysisReceipt, private plan: NumericPlan }
```

Only `SourceTransaction::prepare_shared_analysis(...)` may construct `SharedAnalysisReceipt`. All fields and constructors remain private to the transaction module. Consumers accept borrowed receipt types rather than raw arrays. `NumericPlanReceipt` can only be constructed from a borrowed `SharedAnalysisReceipt` after all six plan blocks finish.

`SharedAnalysisInner` carries the required semantic fields above plus the `SourceStageKey`, the staged analysis-after state owned by the parent, and `advancement_count=1`. The parent compares the active attempt/source/owner/generation before every consumer call. The numeric plan keeps the same shared object; local runtime equality can use private object identity (`Arc::ptr_eq`) and the key. External code cannot forge it because fields and constructors are private.

This wrapper is explicitly LOCAL. It does not assert native layout, native receipt objects, pointer addresses, or native state hashes.

The GTC already requires owner before/after hashes and a canonical shared-receipt digest for transaction evidence. Reuse those existing transaction projections if they are implemented. Do not create an additional F09/F10 digest hierarchy. If the current Rust transaction layer does not yet materialize the existing GTC projection, that is local receipt/verification plumbing still to implement; it is not missing native evidence. Same-object private receipt identity is sufficient for the runtime consumer join, while the existing GTC projection remains the acceptance record.

## Consumer and commit rules

- `prepare_shared_analysis` is called once per SourceTransaction. A second call for the same staged source returns `ANALYSIS_DOUBLE_ADVANCE` before mutating anything.
- F06 and F08–F11 receive the same `&SharedAnalysisReceipt`; F12 receives only `&NumericPlanReceipt` plus codec/BSI/payload inputs.
- The analysis clone stays parent-owned. Consumers cannot install it into committed state.
- F06 failure after F07 staging discards the clone; committed analysis generation and identity remain unchanged. The approved J02 vector records exactly this case.
- A successful source makes analysis, DRC, codec, metadata, clocks, and any published ledger visible in one parent root swap. Top-level commit count is exactly one.
- A failed q candidate cannot alter the shared receipt, numeric plan, or analysis clone. It discards only BAP/group/bitwriter scratch.
- Full restart increments attempt and reconstructs cold owners. Receipts from an earlier attempt cannot pass the new `SourceStageKey`.

## Gap classification

| Item | Status | Classification |
|---|---|---|
| F07 history/transient algorithms and state fields | Defined | No research gap |
| Installed/delayed domains, short flags, spectra | Defined | No research gap |
| F09/F10 plan fields, shapes, and single execution | Defined | No research gap |
| F12 fixed-plan consumption and one source commit | Defined | No research gap |
| Source/attempt/owner/generation binding semantics | Defined by GTC | No research gap |
| Concrete Rust `SharedAnalysisReceipt` type | Not prescribed/absent per implementation report | Local constructor/API work |
| Concrete Rust `NumericPlanReceipt` adapter | Not prescribed/absent per implementation report | Local adapter/API work |
| Private same-object consumer signatures and double-advance guard | Not wired per implementation report | Local orchestration work |
| GTC receipt projection/hash serialization | Contract exists; implementation status not inspected in this research thread | Local transaction verification plumbing if absent |
| Native receipt/state-hash lookup | Not required | Do not investigate |

## Conclusion

There is no algorithm or native field blocker at F09/F10→F12 identity. The minimum implementation is one parent-owned private shared result, one derived numeric plan, borrowed consumer APIs, and one parent root swap. No new digest system, generic receipt framework, or DLL work is justified.

## Wash implementation scope clarification

The private wrapper sketch is one optional LOCAL implementation, not a required Arc allocation or generic receipt framework. Reuse existing staged-owner types and borrowed immutable values when they enforce the same unforgeable construction, source/attempt/generation/provider binding, single F07 advance and parent commit rules. No new user input or native identity is required. This note does not waive existing GTC acceptance projections or authorize premature partial-owner commits.
