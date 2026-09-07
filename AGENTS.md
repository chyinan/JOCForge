# JOCForge

Last verified: 2026-09-07

## Tech Stack

- Rust 2024 workspace with `jocforge-core` and `jocforge` CLI.
- Standard-library core; FFmpeg is an external E-AC-3 carrier executable.
- OpenJOC v0.16.0 is the pinned external validation/decoder/renderer reference;
  identity is enforced by `validation/openjoc-validator.json`, its frozen
  manifest under `D:\Programs\JOCForge-validator`, and the resolver helper.
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
  plus generalized source-derived factorization with bounded analysis/ridge
  and deterministic carrier-causality probes.
- `crates/jocforge-core/src/reduction.rs` — pure deterministic authored-object
  reduction, source accounting, spatial aggregation, and stability metrics.
- `crates/jocforge-core/src/profile.rs` — mandatory idx0/idx1/idx2 topology
  contracts, including profile-owned D0 source indices.
- `crates/jocforge-core/src/rate.rs` — deterministic clean I0/D0 budget
  allocations and legal-rate policy.
- `crates/jocforge-core/src/validation.rs` — pure controlled and adversarial
  temporal corpus generation and metrics.
- `crates/jocforge-core/src/spectral.rs` — pure deterministic spectral/phase
  corpus generation, compatibility metrics, and final-render gate evaluation.
- `crates/jocforge-core/src/eac3.rs` — public fail-closed E-AC-3 audio-block
  walker, nominal-frame padding, skipfld insertion, and CRC2 rewrite.
- `crates/jocforge-core/src/writer.rs` — pure JOC/OAMD/EMDF and independent
  `JocCarriage`/`EmdfProfile` packaging for ETSI and I0 skip-field paths.
- `crates/jocforge-cli/src/main.rs` — file/process orchestration shell.
- `crates/jocforge-cli/src/file_scan.rs` — seek-aware BW64 chunk scanner that
  reads metadata after large `data` chunks without loading PCM, plus bounded
  PCM-frame reads for large programmes.
- `scripts/` — reproducible external validation and evidence generation;
  `final-render-quality.ps1` measures actual decoded/rendered PCM and all
  authoritative OpenJOC callers resolve the frozen manifest artifact.
- `validation/openjoc-validator.json` — authoritative external validator pin.
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
- Carriage and EMDF signaling are explicit: `etsi`/`i0-skipfld` selects
  topology, `etsi`/`deployed` selects signaling, and the default is ETSI
  strict. Deployed compatibility is limited to idx0/1024 initially.
- Matrix and metadata ordering is explicit and stable; unsupported ADM input
  is rejected rather than silently flattened.
- DirectSpeakers LFE routing uses only public `speakerLabel`/`frequency.lowPass`
  semantics; human-readable names never determine routing.
- The static CHNA ingest policy rejects duplicate/conflicting TrackUID,
  track-index, and pack relations before source expansion.

## Contracts and Boundaries

- `profile_spec` is the single JOCForge-owned source for the three mandatory
  public topology labels and dependent channel maps.
- `parse_bw64_adm_metadata` accepts only the static full-track subset, validates
  CHNA one-to-one bindings, and derives LFE from semantic ADM metadata.
- `package_source_derived_access_unit` accepts an explicit EMDF profile, one legal even-sized FFmpeg I0
  frame and two 1536-sample source D0 carrier windows for the selected active
  coded-object set, returning I0 plus a JOCForge-owned PCM-derived D0 carriage
  whose validated minimum is 2560 bytes. Matrix and OAMD row counts must match.
- `package_i0_skip_field_access_unit` accepts a public FFmpeg I0 core, pads
  the I0 skip-field MVP's 896 kbps encoded core to a nominal 4096-byte/1024
  kbps frame, and inserts profile-selected OAMD/JOC EMDF into public
  audio-block skip fields; it emits no D0 and fails closed when capacity is
  insufficient.
- `reduce_scene` and `reduce_scene_with_previous` provide the pure N-to-15
  clean decision boundary. Every supported authored source has a deterministic
  coded-object assignment and positive accounting gain; previous representatives
  and the project-owned temporal continuity cost stabilize source membership;
  `Other` ADM kinds are rejected by the CLI.
- `authored_source_assignment_changes` measures authored-source to coded-slot
  changes; `representative_source_changes` measures representative-source
  identity replacements; `coded_slot_replacements` measures same-slot
  representative changes. Their lifetimes are separate named metrics and must
  not be conflated.
- The CLI reads large BW64 PCM one public 1536-sample frame at a time and pads
  only the final short frame with zeros, preserving source timing/accounting.
- Production code does not import OpenJOC crates or share its serialization
  implementation; external OpenJOC inspect/decode/render remains an
  independent validation loop.
- `spectral_phase_report` and `quantization-stress` provide independent clean
  evidence; `clean-validation-closure.json` is output-only and never an
  acceptance authority.
- `carrier-causality.json` and `source-causality.json` are required evidence
  for idx0/idx1/idx2 intended carrier source coverage; presence/count alone is
  not sufficient.
- D0 source indices are owned by `profile_spec` and consumed by both the
  production package path and the D0 causality evidence; the closure asserts
  the per-row carrier/source identities and both dependent inputs.
- `spectral-phase OUTPUT.json --final-render-report REPORT.json` records the
  internal corpus diagnostics together with a passing pinned
  `FINAL_DECODED_RENDER` evidence stage. The closure runner's PASS predicate
  consumes the Rust final-render gate for actual OpenJOC-rendered PCM.
- `final_render_leakage` is measured from unintended-to-intended channel energy
  in actual frozen-OpenJOC rendered PCM; source-stage
  `unexplained_cluster_leakage` is a separate metric and cannot satisfy it.
- Bitrate support is a `CARRIAGE x PROFILE x BITRATE` contract. ETSI-strict
  supported cells are idx0/idx1/idx2 at 1024, 1280, 1408, 1536, and 1664 kbps;
  deployed-compatible initially supports idx0/1024 only. Lower strict rates
  reject for paired I0/D0 capacity.
  The 384 kbps strict I0 path disables
  automatic FFmpeg channel coupling for pinned OpenJOC traversal correctness.
- No proprietary Dolby encoder binary, media, or reverse-engineered logic is
  permitted in this repository.
- Generated audio/binary evidence under `artifacts/` is local and ignored;
  reproduce the baseline with `scripts/run-mvp.ps1` or the generalized clean
  evidence with `scripts/run-source-derived.ps1` / `scripts/run-decision-engine.ps1`.

## Key Decisions

- The overnight MVP uses a narrow Rust BW64/axml/chna subset because this host
  lacks an exposed CMake/compiler/Boost path; `libadm`/`libbw64` migration is
  follow-up work.
- FFmpeg remains the external public E-AC-3 audio-core encoder. ETSI strict
  retains the established 384+640 I0/D0 allocation at 1024 kbps. The public
  deployed compatibility preset uses the I0 skip-field idx0/1024 carriage,
  encodes a 896 kbps I0 core, pads it to a nominal 4096-byte frame, and sets
  only payload 11/14 `codecdatae=0`; this is a carriage/signaling
  implementation, not a new perceptual E-AC-3 encoder.
- `ReductionConfig.temporal_continuity_weight` defaults to the clean-side
  dimensionless value `0.15`; it adds a switch cost only when a source moves
  between prior-frame clusters and is covered by controlled/adversarial tests.
- DirectSpeakers channel bindings are expanded from ADM references. Non-LFE
  channels use the documented positioned-source fallback; LFE channels are
  carried once in the core LFE path, bounded by the shared 0.90 peak limit, and
  receive explicit accounting records.
- Full matrix, one coarse parameter band, fixed basis continuity, and a
  deterministic 64-sample coefficient analysis view are intentional clean
  milestone limits, not claims of perceptual encoder parity.
- The generalized reducer uses independent signal-signature, spatial, and
  energy weighting with a hard coded-object ceiling of 15. Coded cluster
  positions are energy-weighted aggregates; DirectSpeakers are handled only by
  the explicitly documented positioned-source fallback, not as complete bed
  semantics.
- Final decoded-render checks use the pinned OpenJOC output and independent
  FFmpeg decode as a same-carrier alignment reference. Their structural
  thresholds allow E-AC-3/downmix loss while rejecting non-finite samples,
  spectral collapse/tilt, phase collapse, polarity inversion, excessive
  leakage, and headroom violations; they do not claim perceptual parity.

## Post-D8 generalized profile closure

- `JocProfile` and `profile_spec` cover public ETSI TS 103 420 idx0..idx4.
  idx3 and idx4 are signaling-only phase variants that reuse the idx0 and
  idx2 carrier families; no additional PCM phase transform is emitted.
- `crates/jocforge-core/src/plan.rs` resolves immutable profile, carriage,
  EMDF, and bitrate combinations before ADM I/O. Unsupported combinations fail
  closed with profile-aware errors.
- `docs/joc-capability-matrix.md` and the generated CSV/JSON artifacts define
  the multidimensional profile x carriage x signaling x block x dependent x
  bitrate x container lattice. idx5..idx7 remain reserved and never emit.
- Canonical ETSI raw-elementary idx0..idx4 cells at 1024, 1280, 1408, 1536,
  and 1664 kbps have five-profile OpenJOC/FFmpeg/render/causality evidence;
  the LFE-present variants have a separate five-profile evidence run.
- General short-block, multi-dependent, CMAF, and legacy-core generation is
  classified explicitly in the capability matrix and remains fail-closed until
  independent writer and validation contracts are added.
- The validated advanced legacy-core exception is `legacy-ac3` with Flat-7.X
  idx1, 1280 kbps, one original-syntax AC-3 I0 plus one E-AC-3 D0; other
  legacy-core cells remain fail-closed.
- `crates/jocforge-core/src/conformance.rs` provides the pure public contract
  for 1/2/3/6-block grouping to one 1,536-sample interval and sequential D0..D7
  ID bounds; it is a planning/validation API, not a compressed short-frame
  serializer.
