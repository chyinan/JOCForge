# Agent-D clean-room remediation record

> Date: 2026-09-02
> Scope: clean reconstitution and closure of the Agent-D findings recorded in
> the prior audit outputs. No encoder feature expansion is part of this work.

## Repository reconstitution

The approved clean ref universe was enumerated as the single local ref
`refs/heads/master` at
`1f4c44bfd1a508f299e1aa7fd3c825b759103151`, with tree
`09488fab8357e63bd1b053691ae6b692422f2bc4`. A bundle containing only that ref
was verified as a complete history and cloned into a new object database. The
former worktree was not part of this release candidate; its old `.git` is not
in the implementation path. The clean working path now has only the local
`master` ref; its initial unreachable-object check was empty. Only clean-side
history and public specifications are inputs to this implementation.

## Important findings

| ID | Affected code/docs/test | Missing evidence or implementation | Permitted clean remediation | Acceptance evidence |
| --- | --- | --- | --- | --- |
| F-002 | `scripts/run-validation-closure.ps1`, `docs/quality-gates.md`, metrics sidecar | Leakage was written as zero from merge labels; compatibility had only process exit status | Recompute coded-signal residual, factorization residual, and speaker projection from clean PCM/matrix data; apply finite published thresholds and fail closed | `leakage-matrix.csv`, `clean-validation-closure.json`, T-007/T-008, threshold table |
| F-003 | `crates/jocforge-core/src/adm.rs`, encode admission, ADM tests | First block was selected; extra trajectories and non-full timing could be silently flattened; programme membership was not enforced | Accept only one static full-track block/programme shape; reject extra blocks, unsupported jump positions, non-zero runtime/start, non-full duration, ambiguous/missing encode programme membership | parser rejection tests, explicit diagnostics, supported fixture encode |
| F-004 | `crates/jocforge-core/src/reduction.rs`, reduction tests | Continuity state was conceptually tied to array offsets rather than source identity | Store and look up prior assignments as `(source_identity, coded_index)` pairs | identity helper test and sparse/reordered source regression test |

## Mandatory partial requirements

All 15 mandatory partial rows from the Agent-D matrix are listed here without
downgrading them to research questions.

| Requirement | Affected code/docs/test | Missing evidence or implementation | Permitted clean remediation | Required acceptance evidence |
| --- | --- | --- | --- | --- |
| PUB-003 | `adm.rs`, CLI metadata admission, ADM tests | timing/trajectory semantics were retained but not applied | define and enforce the static full-track subset; reject unsupported forms | timing/block/programme parser tests |
| PUB-004 | `adm.rs`, CLI rejection path | extra block and trajectory forms were not rejected | reject every unsupported ADM form before PCM encoding | explicit lowercase error and rejection tests |
| RED-003 | `adm.rs`, reduction-source boundary | source timing could be ignored | accept only full-track timing or reject it | object/block timing tests and source accounting |
| RED-007 | `reduction.rs`, closure script, quality gates | leakage result was tautological | recompute coded signal from source membership/gains and threshold the residual | `unexplained_cluster_leakage <= 1e-12` in closure evidence |
| CAR-003 | `analysis.rs`, CLI sidecar | no closed quality-budget residual | measure the actual carriers×matrix residual under a structural threshold derived from the fresh five-carrier clean baseline | factorization oracle and T-008/bitrate evidence |
| CAR-006 | CLI sidecar, temporal quality gates | carrier/matrix discontinuity was not measured | record consecutive factorization carrier/matrix deltas | finite `matrix_coefficient_jump`/`carrier_coefficient_jump` bounds |
| MAT-004 | `analysis.rs`, reduction, closure script | signal/spatial existed but leakage/compatibility were research-only | publish and enforce clean residual/projection gates | closure JSON with all required metric fields |
| MAT-006 | `analysis.rs`, CLI sidecar | deterministic output existed without coefficient-jump evidence | calculate finite element-wise consecutive coefficient deltas | analysis jump test and sidecar fields |
| TMP-003 | reduction, analysis, temporal docs | writer coefficient continuity was unmeasured | preserve identity-keyed reduction state and measure matrix/carrier jumps | sparse identity test plus temporal sidecar |
| TMP-004 | ADM parser, reduction, OAMD docs | input trajectories could be flattened | reject non-static trajectory/timing forms; retain deterministic aggregate positions for accepted input | ADM rejection tests and OAMD position evidence |
| TMP-005 | reduction continuity state and tests | sparse source identity lookup gap | use identity-keyed prior assignment state | sparse-index regression and separate identity metric |
| TMP-006 | CLI frame loop, closure script, CLI tests | short frame existed but repeated-process evidence was absent | run a short-final-frame encode twice and verify ceil count/no duplicates/hash | `short-final-process.json`, T-011 |
| COM-003 | closure script, quality gates, profile runs | render success had no numeric speaker error | compute clean left/right speaker projection error and retain OpenJOC render status | per-profile `speaker_render_error`, thresholded T-008 |
| COM-004 | closure script, quality gates | no numeric coloration/leakage/phase-compatible proxy | enforce finite speaker/factorization proxies and document their non-perceptual scope | closure thresholds, OpenJOC/FFmpeg logs, T-008 |
| OAMD-002 | ADM parser, reduction, OAMD writer/tests | aggregate position existed while input trajectory coherence was unguarded | apply aggregate position only to accepted static blocks; reject unsupported trajectories | static-boundary tests and OAMD output evidence |

## Minor findings

- F-005: removed the unused legacy authored-object admission API and its
  contradictory `>15` error. Active encode admission is now programme-aware;
  N-to-15 reduction remains the current policy.
- F-006: named the numerical floors, tie/pivot tolerances, ridge controls, D0
  feature shape, OAMD coordinate mapping, and metadata bound; their clean
  rationale and boundary behavior are recorded in code comments, this
  document, and the related tests/closure fields.
- F-007: `run-validation-closure.ps1` is the authoritative acceptance runner.
  `run-decision-engine.ps1` now labels its older sampling statuses as deferred
  to that runner rather than presenting a competing acceptance matrix.
- F-008 (INFO): public profile labels, downmix indices, channel maps, and
  seventh-channel ordering remain covered by `profile.rs` tests. A stronger
  decoded semantic comparison is intentionally non-blocking and is not part
  of this clean remediation.

## Evidence boundary

The residual and speaker metrics are generic mathematical structural checks.
They do not claim proprietary encoder parity, transparent perceptual E-AC-3
encoding, patent clearance, or production perceptual quality. The final report
must still include fresh command output for the repository object database,
source/docs/tests/history scans, Rust gates, all clean validation scripts, and
the three public profile strict checks.
