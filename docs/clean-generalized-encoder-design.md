# Clean generalized JOC encoder design

> Status: implementation baseline for Agent C
> Date: 2026-09-02

## Scope and firewall

This design uses only the repository, the files in the sanitized
`clean-handoff` directory, public ETSI/ADM/BW64 semantics, and generic signal
processing. It does not depend on private encoder identity, private mappings,
private thresholds, or reference output bytes.

The public JOC contract describes `joc_num_objects` as a one-based six-bit
field and uses `b_joc_obj_present` for each declared object. The implementation
therefore emits the actual active coded-object count rather than declaring
unused objects. The selected profile remains the source of truth for the
five-channel idx0 topology and the distinct seven-channel idx1/idx2 topologies.

## Design alternatives considered

1. **Existing low-rank basis only.** Reuse the first K source signals as
   carriers and solve a matrix. This is small, but it does not decide which
   authored objects share a coded representation and can fail on correlated
   or rank-deficient inputs.
2. **Iterative k-medoids with temporal state.** Optimize assignments repeatedly
   against audio and position features. This can improve a measured objective,
   but adds convergence, tie-breaking, and identity-transition complexity
   before the generalized path is validated.
3. **Deterministic farthest-point seed selection plus one assignment (chosen).**
   Select high-energy/spatially separated source seeds, assign every source to
   the closest seed using normalized signal and spatial features, and combine
   each cluster with explicit energy weights. Seed order is source-ordinal
   order after selection, so identical inputs produce identical identities.

The chosen approach is intentionally incremental: it provides a real N-to-M
decision engine and complete source accounting without asserting perceptual
parity. A later clean experiment can replace the reducer behind the same
result contract.

## Core data flow

```text
ADM objects + PCM tracks
        -> validated ReductionSource values
        -> deterministic seed selection and source-to-coded mapping
        -> bounded weighted coded-object signals + aggregate positions
        -> source accounting and quality diagnostics
        -> profile-specific carrier factorization
        -> public quantized full matrix + OAMD/EMDF
        -> imperative CLI / FFmpeg / external validators
```

The reduction result is pure data. File reads, FFmpeg invocation, temporary
files, output writes, and OpenJOC/FFmpeg validation remain in the CLI or
scripts.

## Clean-side objective and safeguards

The default reduction objective is a weighted sum of normalized source
signature distance and spatial distance, with a smaller log-energy distance
term used to avoid grouping only by loudness. The default weights are
project-owned parameters: signal `0.55`, spatial `0.35`, and energy `0.10`.
The signal signature uses 16 deterministic window means; this is a compact
whole-programme feature, not a claim about a codec analysis cadence.

Each cluster uses positive RMS-derived weights with a deterministic floor for
silent sources. Its representative position is the same weight family's
weighted centroid. The combined signal is scaled only when its absolute peak
exceeds the configured `0.90` headroom target. Every source record includes
its cluster index, positive contribution gain, represented energy, relative
error, and spatial error; no source is discarded silently.

For N <= the active budget, the reducer keeps one coded representation per
source. For N > budget, it selects exactly the configured budget of seeds and
assigns all sources. The initial budget is 15, checked both by the reducer and
by the writer boundary.

## Carrier and matrix policy

The public profile fixes carrier count and ordering: idx0 uses five channels;
idx1 uses `L,R,C,Ls,Rs,Lrs,Rrs`; idx2 uses
`L,R,C,Ls,Rs,Tfl,Tfr`. Carriers are source-derived combinations of reduced
signals. If there are fewer reduced objects than carrier channels, the extra
carriers are deterministic source-derived blends and the matrix contains zero
for unused dimensions.

The first production representation is full matrix, one parameter band, one
data point, and the public coarse 96-symbol JOC Huffman code. Coefficients are
quantized before serialization and rejected if they leave the public domain.
The public coarse Huffman table is included as a standards-derived table in
the writer; it is not an inferred reference implementation table.

## Temporal and compatibility policy

The reducer preserves source ordinal order for unmerged scenes and stable seed
ordering for merged scenes. A pure `compare_reduction_assignments` diagnostic
reports changed source assignments between consecutive results. The initial
CLI processes a complete programme, while tests can invoke the same pure
reducer on consecutive windows. OAMD uses aggregate position for merged
sources and documents the narrow DirectSpeakers policy: the parsed component
is either retained as an explicitly classified source contribution or rejected
with a named unsupported-input error; it is never silently omitted.

Compatibility mode remains profile-fixed in this milestone. The CLI records
the selected profile, carrier count, active coded-object count, matrix and
reduction metrics. Speaker-render comparison is an external validation step;
the implementation does not claim production perceptual quality.

## Verification plan

- Pure core tests cover budget, non-empty source accounting, bounded output,
  determinism, stable assignments, finite diagnostics, and invalid input.
- Writer tests cover actual object count, inactive-slot removal, coarse Huffman
  zero/positive/negative symbols, and all three profile dimensions.
- CLI/script tests cover the controlled corpus, rate and framing diagnostics,
  deterministic re-encode, OpenJOC strict validation, FFmpeg decode, finite
  PCM, and the 28-object Hanabi acceptance path where local inputs/tools exist.
- Every clean requirement receives an explicit traceability disposition in
  `docs/clean-implementation-traceability.md`.
