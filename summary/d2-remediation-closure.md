# D2 remediation closure summary

Verified 2026-09-03 from the current clean-side worktree.

## Closure map

- `adm.rs`: DirectSpeakers LFE derives from `speakerLabel`/`frequency.lowPass`, recognizes public `RC_LFE`/`RC_LFE2`, rejects missing/conflicting semantics, and validates CHNA one-to-one rows before expansion.
- `main.rs`: LFE channels are summed once, passed through the shared 0.90 peak limit, and recorded with measured peak/headroom metrics.
- `spectral.rs`: ten deterministic controlled fixtures measure magnitude, band energy, tilt, transient retention, phase, coherence, polarity, quadrature, width, and boundary behavior; the 16-source reduction row is supplemental.
- `run-validation-closure.ps1`: lower-rate commands execute for real; T-001/T-004/T-008/T-009 derive statuses from primary measurements; closure JSON is output-only; the anti-self-report meta-test mutates only temporary JSON.
- `openjoc-validator.json` plus pin scripts: OpenJOC 0.16.0, source revision `ad6556babf42566f1a09820b01dc333703c8b1da`, and executable SHA256 are enforced before validation.
- `audit-git-release.ps1`: master-only ref allowlist, extension/reflog/object checks, and clean-side history scans are reproducible.

## Verified evidence

The closure runner regenerated `clean-validation-closure.json` with 12/12
PASS, zero failures/not-run, a passing anti-self-report test, passing
quantization stress over all 96 symbols and 2,280 matrix values, and passing
ten-fixture spectral/phase hard gates. Hanabi temporal stabilization retained
39,503→140 assignment-change reduction, zero coded identity changes, all three
profile strict/decode checks, deterministic repeat, and one singular semantic
LFE path.

The current Git audit remains intentionally pre-cleanup FAIL while the
application snapshot refs, reflogs, and baseline administrative path references
remain in the existing checkout/history. Final repository reconstitution and
temporary-progress cleanup are the remaining release-gate actions.
