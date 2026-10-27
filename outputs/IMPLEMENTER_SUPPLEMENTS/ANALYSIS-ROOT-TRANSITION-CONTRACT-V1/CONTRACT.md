# Analysis root transition contract

## Decision

The selected standalone `analysis` GTC root is fully derivable from the approved GTC, F07–F12 composition, state schema, provider manifest, and predecessor shared-analysis crosswalk. No model or DLL lookup is required.

The unresolved work reported by implementation is local construction and wiring: create the committed/cold analysis root, clone it for one source, issue one parent-owned shared analysis result after F07, finish F08–F10 on that same staged root, and install it only through the parent transaction. No algorithm field or provider field is missing.

## Canonical root boundary

The standalone analysis root contains only the F07–F10 persistent families:

- input history: generation plus 7,680 F32 bit words;
- transient state: generation; 20 two-section history words; 30 three-section history words; 90 peak-statistics words; 5 previous-mean words; 20 short-queue I1 values; 20 other-queue I1 values;
- primary/companion/LFE transform state: generation; 640 primary FBW, 640 companion FBW, 128 primary LFE, and 128 companion LFE F32 bit words;
- policy state: generation; 64 gain/db history words, 30 exponent-refresh U16 words, and 30 normalized-anchor U16 words.

It does not contain source lifecycle, codec transform/group/bitwriter state, `previous_short`, payload FIFO, publication count, DRC state, or metadata state. Those remain separate GTC owners.

The exact machine-readable field names and dimensions are in `ANALYSIS-ROOT-SCHEMA.json`. It preserves the existing `F07_F12_OWNER_STATE_V5` field vocabulary and canonical scalar domains. This supplement adds no state hash or canonicalization algorithm.

For every committed analysis root, the four family generations are equal. The GTC analysis owner receipt's `before_generation` and `after_generation` are that common committed generation. Partial staged roots may temporarily have unequal family generations and are never commit-eligible.

## Cold and reset

Cold analysis state has all four generations equal to zero. Every U32/U16 history word is zero and every I1 queue value is zero. F32 zero is the positive-zero bit word `0x00000000`.

Full reset destroys the old attempt root, increments attempt, and constructs this exact cold value again. `reset_state == cold_state` field-for-field. Source 0 replay after reset must reproduce the first cold source-0 core result. Reset is not performed at source 2 publication start and no analysis-only EOF flush exists.

Authority: F07 reset (`specs/07-history-and-transient.md:104-118`), F08 cold/reset (`specs/08-transforms.md:23,108-114`; `specs/08-nonzero-transform-composition.md:39-44`), source composition reset (`specs/12-selected-source-frame-composition.md:62-70`), and the approved cold/reset result.

## One-source transition

Let the committed root before source `s` have common generation `G`.

### Parent prepare

The parent SourceTransaction first validates the active attempt, transaction, source, analysis owner identity, and committed generation. It clones the committed root; committed state is not mutated.

`SharedAnalysisPrepare` then runs F07 once:

- input history generation becomes `G+1` in the clone;
- transient generation becomes `G+1` in the clone;
- transform and policy generations remain `G`;
- delayed audio, short/other flags, structure refresh, and next F07 state are produced once;
- the parent issues one immutable shared result with `advancement_count=1`, `before_generation=G`, `staged_generation=G+1`, and the same source/attempt/owner/provider identity.

This `F07_PREPARED` clone is not commit-eligible because transform and policy still belong to `G`.

### Numeric plan completion

F08 consumes the delayed audio and short flags from that same shared result. It uses the selected F08 provider and advances primary/companion/LFE histories once, so transform generation becomes `G+1`.

F09 and F10 consume the spectra from that same source and provider lineage, build POWER/PRIMARY, gain/db, refresh/strategy, normalization, expanded exponents, and converter inputs once, and advance policy generation to `G+1`.

Only after all six blocks complete may the parent derive a NumericPlan result. At `NUMERIC_PLAN_COMPLETE`, all four staged family generations equal `G+1`; the result references the exact shared analysis object and carries the same attempt/source/owner/provider identity. F12 may read it but cannot advance F07–F10.

### Commit or discard

F12 candidate retries mutate only candidate scratch. A failed candidate cannot mutate the analysis clone or either shared result.

If any stage fails, the entire staged analysis clone is discarded and the committed analysis root remains byte-identical at generation `G`. If the full source succeeds, the parent CallbackTransaction installs the staged analysis root together with every other owner in one root swap; analysis becomes committed generation `G+1`. No analysis consumer performs an independent commit.

Authority: GTC `specs/00-global-transaction-contract.md:118-137,143-153`, cross-stage join `specs/00-cross-stage-contract-joins.md:5-7`, and source composition `specs/12-selected-source-frame-composition.md:13-28,34-54,88-99`.

## Parent receipt semantics

The parent-issued shared result must semantically bind:

- attempt and transaction identity;
- source index;
- analysis owner identity;
- committed before generation `G`;
- staged generation `G+1`;
- `advancement_count=1`;
- installed/delayed domain identity and immutable data/reference;
- 30 short flags and the primary unquantized spectrum;
- selected provider-set identity.

This is a semantic contract, not a new serialized receipt ABI. The local implementation may use the predecessor's private constructor/shared-object choice. Existing GTC owner before/after hashes and the existing canonical receipt projection remain the acceptance evidence; this supplement does not add another digest.

## Provider lineage

The selected analysis provider set is complete:

| Stage | Provider identity | Required use |
|---|---|---|
| F07 transient | `transient-d05-authorized-v1`, SHA-256 `130cd1a446ea8413a1d0d4c5eea631e890b7f1cd675975efa92697e258402991` | 33 reviewed F32 parameters |
| F08 transform | `ordinary-transform-math-v1`, SHA-256 `f1fb1a16b3caedab6ddc583201b9454755fd8871ecdf7881a6a3ffbad10fddf1` | selected generated window/rotation/permutation/LFE provider |
| F09/F10 policy | `policy-authorized-v1`, SHA-256 `a99d077900058eb4a862f5ea29bc87408bf6d7a4f4cc42e3da66ae6c68a93c6a` | 19 reviewed policy scalars |
| F09 difference map | `exponent-refresh-difference-math-v1`, SHA-256 `6cf3ae8d899b334d5aa87e0744e4c58b7dba1d92177ca07738726f8ea0ee2c6d` | 32 reviewed I16 entries |

The F08 selected provider intentionally differs from historical research identity in 12 window words and some LFE trigonometric values. That difference is declared by the selected provider and is not a missing field. Real encoded/render acceptance remains downstream, but it does not reopen provider lineage for this root contract.

## Same-source F09/F10 join

The NumericPlan result must retain the exact shared analysis identity and source. It contains all six-block completion status, short/other flags, primary/companion/LFE spectra, POWER snapshot, PRIMARY raw exponent rows, gain/db, strategy, normalized anchors/groups, expanded exponents, converter result, and provider identities.

F09/F10 execute once on the staged analysis root. Candidate q changes only BAP/mantissa/capacity scratch and cannot rebuild or modify the plan. Source, generation, provider, or shared-result mismatch is rejected before F12 numeric work. A second F07 call is `ANALYSIS_DOUBLE_ADVANCE`.

## Gap classification

- Analysis root fields and dimensions: complete.
- Cold/reset values: complete.
- F07 staged/committed generation transition: complete.
- Parent `advancement_count=1` semantics: complete.
- F08 provider lineage: complete for selected provider.
- F09/F10 same-source plan join: complete.
- Concrete Rust root constructor, shared-result constructor, and adapters: local implementation work if absent.
- New native/model evidence: not required.
- New digest, receipt serialization ABI, generic state framework, or F12 algorithm: out of scope.

## Limitations

This is a path-free Wash supplement. It does not inspect implementation source, prescribe module/file names, define memory layout, claim native receipt equivalence, execute the DLL, rerun real audio, or establish render/listening acceptance. Input-only transitions provide control/state recipes, not expected spectra, hashes, frame bodies, or PCM.
