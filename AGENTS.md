# JOCForge

Last verified: 2026-10-15

Read `C:\Users\chyinan\.codex\RTK.md`. Prefix shell commands with `rtk`; run PowerShell through `rtk proxy pwsh -NoProfile -File ...`.

This is a migration scaffold. Encoding is unavailable. Read docs/architecture before editing. ARCHITECTURE_REVIEW=PENDING and IMPLEMENTATION_READY=NO. No audio, JOC or E-AC-3 algorithms may be implemented until the human approves architecture and provides the approved specification and GTC owner projection. Use only that approved package; historical code, Git history, migration audits, archived vectors and research materials are excluded as implementation sources.

domain is pure; application coordinates domain; io owns file adapters; CLI owns process entry; validation owns independent external comparisons. Keep state private to one approved owner. Cross boundaries with immutable typed receipts, proposals or staged results. Add Functional Core or Imperative Shell classification comments. No mutable globals, algorithm feature forks or utility dumping grounds.

Run fmt/check/clippy/test with --locked and scripts/check-boundaries.py. Tests in this baseline establish rejection and engineering boundaries only, never codec conformance. No proprietary encoder inputs are permitted. If agents are delegated, wait for results; do not terminate them merely after 30 seconds.
