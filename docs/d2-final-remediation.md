# D2 final remediation traceability

> Date: 2026-09-03
> Scope: clean-side remediation of the D2 audit findings. This document does
> not claim the Agent-D3 provenance verdict.

The exact D2 blockers are enumerated in
`analysis/d2-remediation-matrix.csv`. The machine-readable D2 audit identified
six Important findings and one Minor finding. The five unresolved mandatory
requirements were PUB-003, PIPE-001, PIPE-004, CAR-004, and COM-004. The two
failing tests were T-008 and T-009.

| D2 finding | Closure implementation/evidence | Requirement/test disposition |
| --- | --- | --- |
| D2-I-001 | `docs/git-release-ref-policy.md` and `scripts/audit-git-release.ps1` define and check a master-only ref/object namespace; final evidence is `artifacts/git-release-audit.json`. | D2 Git release gate closed when zero unexpected/unreachable objects and no reflogs/extensions are reported. |
| D2-I-002 | `scripts/run-validation-closure.ps1` invokes lower-rate CLI rejection probes, derives T-001/T-004/T-008/T-009 from measured primary results, and records measured/expected fields. | Self-reported clean rows are removed; T-009 is derived from `quantization-stress.json`; anti-self-report meta-test is emitted in closure. |
| D2-I-003 | `crates/jocforge-core/src/spectral.rs` provides a ten-fixture deterministic corpus and independent FFT/phase/coherence measurements; `docs/spectral-phase-quality-gates.md` defines the invariant gates. | COM-004 and T-008 are derived from spectral-phase output plus controlled external validation. |
| D2-I-004 | `adm.rs` derives LFE only from public `speakerLabel`/`frequency.lowPass` semantics and rejects missing/conflicting DirectSpeakers semantics; `main.rs` applies the shared 0.90 peak policy and reports headroom. | PUB-003, PIPE-001, and CAR-004 are closed by semantic, adversarial, peak/headroom, and Hanabi evidence. |
| D2-I-005 | `validation/openjoc-validator.json` pins version, source revision, executable SHA256, platform, and commands; `scripts/openjoc-validator-pin.ps1` hard-fails mismatches. | Validator pin test rejects a wrong executable; docs and validation scripts use the same identity. |
| D2-I-006 | `parse_chna` rejects duplicate TrackUIDs, duplicate track indices, empty relations, out-of-range indices, conflicting pack relations, and duplicate ADM references. | PIPE-001/PIPE-004 close with no last-write-wins path; unique/exact/conflicting CHNA tests cover the policy. |
| D2-M-001 | Releasable docs no longer contain historical administrative path identifiers; the Git audit excludes temporary progress files from its clean-side scan and the final worktree removes them. | Strict current-tree/reachable-history contamination scan is reproducible and must be PASS before release handoff. |

## Evidence authority

`clean-validation-closure.json` is output only. Its statuses are generated
after primary commands/tests execute by `scripts/run-validation-closure.ps1`.
Deleting the JSON and rerunning the script reconstructs the statuses from
metrics CSV, command exit codes, quantization measurements, spectral
measurements, and external validation logs. `scripts/run-decision-engine.ps1`
delegates to this runner and refuses to generate a competing matrix when the
closure is absent.

The spectral command intentionally reports its E-AC-3/OpenJOC stages as
`NOT_AVAILABLE`; T-008 pairs its controlled clean compatibility invariants
with actual profile E-AC-3/OpenJOC/FFmpeg structural checks. No production
perceptual-quality claim is made.
