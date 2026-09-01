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

## Project Structure

- `crates/jocforge-core/src/adm.rs` — narrow public BW64/ADM subset and
  deterministic synthetic fixture generation.
- `crates/jocforge-core/src/analysis.rs` — pure rank-controlled factorization
  and explicit matrix quantization.
- `crates/jocforge-core/src/profile.rs` — mandatory idx0/idx1/idx2 topology
  contracts.
- `crates/jocforge-core/src/writer.rs` — pure JOC/OAMD/EMDF and I0/D0 writer.
- `crates/jocforge-cli/src/main.rs` — file/process orchestration shell.
- `scripts/` — reproducible external validation and evidence generation.
- `summary/` — focused investigation summaries.

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
- `package_ffmpeg_access_unit` accepts exactly one 4096-byte I0 frame and
  returns I0 + JOCForge-owned D0 carriage.
- Production code does not import OpenJOC crates or share its serialization
  implementation; external OpenJOC inspect/decode/render remains an
  independent validation loop.
- No proprietary Dolby encoder binary, media, or reverse-engineered logic is
  permitted in this repository.
- Generated audio/binary evidence under `artifacts/mvp/` is local and ignored;
  reproduce it with `scripts/run-mvp.ps1`.

## Key Decisions

- The overnight MVP uses a narrow Rust BW64/axml/chna subset because this host
  lacks an exposed CMake/compiler/Boost path; `libadm`/`libbw64` migration is
  follow-up work.
- FFmpeg encodes the actual 5-channel I0 carrier at 1024 kbps. A fixed,
  standards-derived 2560-byte D0 keeps the raw stream at 1664 kbps.
- Full matrix, one coarse parameter band, fixed basis continuity, and a
  rank-controlled synthetic fixture are intentional MVP limits, not claims of
  perceptual encoder parity.
