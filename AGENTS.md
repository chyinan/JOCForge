# JOCForge

Last verified: 2026-09-02

## Tech Stack

- Rust 2024 workspace with `jocforge-core` and `jocforge` CLI.
- Standard-library core; FFmpeg is an external E-AC-3 carrier executable.
- OpenJOC v0.15.0 is an external validation/decoder/renderer reference only.
- No C++ dependency is linked in the MVP; EBU C++ projects are documented
  public references pending a proportionate FFI integration.

## Commands

- `cargo fmt --all -- --check`
- `cargo check --workspace --locked`
- `cargo clippy --workspace --all-targets --locked -- -D warnings`
- `cargo test --workspace --locked`
- `.\scripts\run-mvp.ps1`
- `.\scripts\run-decision-engine.ps1` — generalized clean corpus, Hanabi,
  OpenJOC/FFmpeg evidence, and clean-test status report.
- `.\scripts\run-validation-closure.ps1` — clean bitrate, DirectSpeakers,
  leakage, temporal and matrix closure evidence.
- `.\scripts\run-temporal-stabilization.ps1` — Hanabi before/after and
  adversarial continuity evidence.

## Project Structure

- `crates/jocforge-core/src/adm.rs` — narrow public BW64/ADM subset and
  deterministic synthetic fixture generation.
- `crates/jocforge-core/src/analysis.rs` — pure rank-controlled factorization
  plus generalized source-derived factorization with bounded analysis/ridge.
- `crates/jocforge-core/src/reduction.rs` — pure deterministic authored-object
  reduction, source accounting, spatial aggregation, and stability metrics.
- `crates/jocforge-core/src/profile.rs` — mandatory idx0/idx1/idx2 topology
  contracts.
- `crates/jocforge-core/src/rate.rs` — deterministic clean I0/D0 budget
  allocations and legal-rate policy.
- `crates/jocforge-core/src/validation.rs` — pure controlled and adversarial
  temporal corpus generation and metrics.
- `crates/jocforge-core/src/writer.rs` — pure JOC/OAMD/EMDF and I0 plus
  PCM-derived dependent D0 writer.
- `crates/jocforge-cli/src/main.rs` — file/process orchestration shell.
- `crates/jocforge-cli/src/file_scan.rs` — seek-aware BW64 chunk scanner that
  reads metadata after large `data` chunks without loading PCM, plus bounded
  PCM-frame reads for large programmes.
- `scripts/` — reproducible external validation and evidence generation.
- `summary/` — focused investigation summaries.
- `docs/clean-generalized-encoder-design.md` and
  `docs/clean-implementation-traceability.md` — clean algorithm decisions and
  71-requirement disposition.

## Conventions

- Application files carry a Functional Core or Imperative Shell classification
  comment.
- Core logic is deterministic and side-effect free; file/process I/O stays in
  CLI or scripts.
- User-facing errors are lowercase actionable fragments.
- Matrix and metadata ordering is explicit and stable; unsupported ADM input
  is rejected rather than silently flattened.

## Contracts and Boundaries

- `profile_spec` is the single JOCForge-owned source for the three mandatory
  public topology labels and dependent channel maps.
- `package_source_derived_access_unit` accepts one legal even-sized FFmpeg I0
  frame and two 1536-sample source D0 carrier windows for the selected active
  coded-object set, returning I0 plus a JOCForge-owned PCM-derived D0 carriage
  whose validated minimum is 2560 bytes. Matrix and OAMD row counts must match.
- `reduce_scene` and `reduce_scene_with_previous` provide the pure N-to-15
  clean decision boundary. Every supported authored source has a deterministic
  coded-object assignment and positive accounting gain; previous representatives
  and the project-owned temporal continuity cost stabilize source membership;
  `Other` ADM kinds are rejected by the CLI.
- `reduction_assignment_changes` measures authored membership changes;
  `reduction_identity_changes` measures representative identity replacements.
  These are separate metrics and must not be conflated.
- The CLI reads large BW64 PCM one public 1536-sample frame at a time and pads
  only the final short frame with zeros, preserving source timing/accounting.
- Production code does not import OpenJOC crates or share its serialization
  implementation; external OpenJOC inspect/decode/render remains an
  independent validation loop.
- No proprietary Dolby encoder binary, media, or reverse-engineered logic is
  permitted in this repository.
- Generated audio/binary evidence under `artifacts/` is local and ignored;
  reproduce the baseline with `scripts/run-mvp.ps1` or the generalized clean
  evidence with `scripts/run-source-derived.ps1` / `scripts/run-decision-engine.ps1`.

## Key Decisions

- The overnight MVP uses a narrow Rust BW64/axml/chna subset because this host
  lacks an exposed CMake/compiler/Boost path; `libadm`/`libbw64` migration is
  follow-up work.
- FFmpeg encodes the actual 5-channel I0 carrier at 1024 kbps. JOCForge writes
  a valid 2560-byte dependent D0 whose two channels are driven by source
  carrier PCM-window features; the D0 path is source-derived but not yet a
  transparent perceptual E-AC-3 encoder.
- `ReductionConfig.temporal_continuity_weight` defaults to the clean-side
  dimensionless value `0.15`; it adds a switch cost only when a source moves
  between prior-frame clusters and is covered by controlled/adversarial tests.
- DirectSpeakers channel bindings are expanded from ADM references. Non-LFE
  channels use the documented positioned-source fallback; LFE channels are
  carried once in the core LFE path and receive explicit accounting records.
- Full matrix, one coarse parameter band, fixed basis continuity, and a
  deterministic 64-sample coefficient analysis view are intentional clean
  milestone limits, not claims of perceptual encoder parity.
- The generalized reducer uses independent signal-signature, spatial, and
  energy weighting with a hard coded-object ceiling of 15. Coded cluster
  positions are energy-weighted aggregates; DirectSpeakers are handled only by
  the explicitly documented positioned-source fallback, not as complete bed
  semantics.
