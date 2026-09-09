# JOCForge license audit

Audit date: 2026-09-09

This is an engineering release audit, not legal advice. It separates the
copyright license for JOCForge source from FFmpeg's license, third-party patent
questions, and trademark questions.

## Decision

Recommended project license: **Apache-2.0**.

JOCForge is an independent Rust implementation based on public specifications
and public, redistributable references. Apache-2.0 permits source and binary
redistribution, modification, commercial use, and sublicensing. It also gives
contributors the standard copyright and patent grants, subject to the patent
termination clause. Those terms fit a permissive codec implementation and a
low-friction contribution model.

Apache-2.0 requires recipients to receive the license, requires notices for
modified files, preserves relevant attribution notices, and propagates
attribution notices from a `NOTICE` file when one exists. It does not grant
permission to use another party's trademarks. JOCForge has no project `NOTICE`
obligation identified in this audit, and the repository does not invent a
copyright holder or add per-file headers.

The patent grant covers patent claims that a contributor can license and that
are necessarily infringed by that contribution. It does not clear third-party
patents, make the codec patent-free, or provide legal immunity. Clean-room
provenance is a separate copyright/provenance control.

## Cargo dependency census

`cargo metadata --locked` and `cargo tree --locked --edges all` agree on this
workspace:

| Class | Count | Packages or edges |
| --- | ---: | --- |
| Publishable workspace packages | 2 | `jocforge`, `jocforge-core` |
| Direct runtime dependency edges | 1 | `jocforge` → local `jocforge-core` |
| Transitive runtime dependencies | 0 | none |
| Build dependencies | 0 | none |
| Development dependencies | 0 | none |
| Registry or git packages | 0 | none |

Both publishable crates declare `license = "Apache-2.0"` through the workspace
metadata. There are no copyleft, LGPL, GPL, non-commercial, source-available,
or unknown Cargo dependencies. The dependency result does not decide the
license of separately executed release tools.

`cargo-deny`, `cargo-about`, and `cargo-license` are not installed on this
host. The result is cross-checked from `cargo metadata --locked`, `Cargo.lock`,
and `cargo tree --locked --edges all`; no registry package remains that would
need a heuristic license report reconciled against source metadata.

## FFmpeg boundary

The Rust CLI uses `std::process::Command::new("ffmpeg")` with structured
arguments to create an E-AC-3 or AC-3 carrier. JOCForge does not dynamically or
statically link FFmpeg libraries, vendor FFmpeg source, distribute an FFmpeg
binary in the source repository, or contain copied or mechanically translated
FFmpeg code. `ffprobe` appears in tests and validation scripts as a separately
executed inspection tool; it is not a Cargo dependency.

The current developer machine has an external `8.1-full_build-www.gyan.dev`
FFmpeg on `PATH`. Its reported configuration enables GPL and version-3 code,
so it is evidence for local validation only and is not the license of JOCForge.

The official release model is a separate Windows x64 bundle containing
`jocforge.exe`, `ffmpeg.exe`, and `ffprobe.exe`. Release CI pins FFmpeg 8.1 tag
`n8.1` at commit
`a65b3bfe9dacc3b20597ef199d0afdd8bc8128e2`, verifies the official source archive
SHA-256
`b072aed6871998cce9b36e7774033105ca29e33632be5b6347f3206898e0756a`, and
builds with no `--enable-gpl`, `--enable-nonfree`, or `--enable-version3`.
The resulting FFmpeg configuration is intended to be LGPL-2.1-or-later. The
bundle keeps FFmpeg license files, the complete source archive, build config,
local-change record, and binary hashes separate from JOCForge's `LICENSE`.

The source pin and configuration are recorded in
[`release/ffmpeg-pin.json`](../release/ffmpeg-pin.json). The source hash and
pin are closed; the resulting executable and ZIP hashes remain pending the
first CI build. No official binary is present in this workspace.

CI writes the actual executable hashes to the bundle `manifest.json`, the
bundle `SHA256SUMS.txt`, and the published `release-artifact.json` plus external
`SHA256SUMS.txt`. The smoke report is published with the artifact and changes
its binary-distribution gate to PASS only after the required semantic checks.
The checked-in pin intentionally keeps binary and ZIP fields null until a real
build supplies those values.

## Notices and attribution

`NOTICE_REQUIRED = NO`: no JOCForge or Cargo dependency NOTICE file requires
propagation. `THIRD_PARTY_NOTICES_REQUIRED = YES` for the binary distribution:
the bundle must identify FFmpeg as a separate executable and include its
applicable `LICENSE.md` and `COPYING.LGPLv2.1` files plus the exact source
archive and build provenance. Those FFmpeg texts must not be merged into the
Apache-2.0 `LICENSE`.

The planned FFmpeg configuration disables external library autodetection, GPL
components, nonfree components, and version-3 components. The CI license check
fails if the built configuration reports any of those enable flags. If a
future feature requires a different configuration, it needs a fresh license
and release audit.

## Contributions and branding

`CONTRIBUTING.md` requires original or public-specification-based contributions,
excludes proprietary or confidential implementation material, and places
accepted contributions under Apache-2.0. No CLA or DCO is required for the
first release; Apache-2.0's normal contributor provisions apply.

JOC, E-AC-3, Dolby, and Dolby Atmos references are descriptive technical terms
where used. The public documentation does not claim sponsorship, certification,
authorization, or official Dolby status. It also does not claim patent
clearance. No logo or other third-party brand asset is tracked, and the
`JOCForge` project name presents no source-license issue found by this audit;
trademark review remains separate.

## Release readiness

Source licensing and Cargo metadata are ready for the selected Apache-2.0
model. The self-contained binary distribution is not yet closed because this
workspace has not run the Windows CI build, generated the final executable and
ZIP hashes, or run the extracted-bundle smoke/semantic QA. The workflow refuses
publication without a separately provisioned pinned OpenJOC validator for the
full semantic QA path.

The addendum overrides a source-only first release. The recommended model is
now the public source repository plus the CI-built self-contained Windows x64
bundle. The source release is also provided. A later FFmpeg version change must
update the pin, source hash, build configuration, binary hashes, and release QA;
upstream `latest` is never selected automatically.

The release workflow pins its GitHub Actions by commit. GitHub's Windows image
and the MSYS2 package set are not byte-reproducible inputs, so the binary gate
requires two clean builds and records exact source/configuration/file hashes
before publication. That reproducibility comparison has not run in this
workspace.

Current status:

```text
JOCFORGE_LICENSE_CLOSURE = PARTIAL
LICENSE_AUDIT = PASS
LICENSE_RELEASE_READINESS = REVIEW_REQUIRED
SELF_CONTAINED_BINARY_RELEASE = PARTIAL
BINARY_DISTRIBUTION_LICENSE_GATE = REVIEW_REQUIRED
PUBLIC_RELEASE_GATE = REVIEW_REQUIRED
```

The remaining items are release execution evidence and the pre-existing clean
object database export/publication review. None is a reason to relicense
JOCForge or to call the project patent-free.
