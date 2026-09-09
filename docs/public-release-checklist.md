# JOCForge public release checklist

Status at audit code state `1349457…`: `REVIEW_REQUIRED`.

## PASS

- [x] Full reachable history scan completed: 53 commits; no reachable secret,
  contaminated material, proprietary implementation derivation, or copyright
  media.
- [x] Current tracked tree has no high-confidence secret candidates, private
  absolute paths, or media files.
- [x] Root and workspace package license declarations are Apache-2.0; zero
  registry/transitive dependencies and zero incompatible/unknown licenses.
- [x] Root `LICENSE` contains the canonical Apache-2.0 text, and both
  publishable crates expose `license = "Apache-2.0"` through workspace metadata.
- [x] License audit selects one project license: Apache-2.0. It does not claim
  patent clearance or legal immunity.
- [x] FFmpeg 8.1 source pin, commit, source hash, LGPL-compatible configuration,
  and `LOCAL_FFMPEG_MODIFICATIONS = NONE` are recorded in
  `release/ffmpeg-pin.json`.
- [x] `THIRD_PARTY_NOTICES.md` defines the separate FFmpeg license/source
  package; no repository-level `NOTICE` file is required.
- [x] `cargo fmt --all -- --check`.
- [x] `cargo check --workspace --locked`.
- [x] `cargo clippy --workspace --all-targets --locked -- -D warnings`.
- [x] `cargo test --workspace --locked`: 173 passed.
- [x] `git diff --check`.
- [x] Isolated fresh clone build/test gates passed.
- [x] Synthetic public smoke passed for idx0..idx4 with OpenJOC and FFmpeg.
- [x] Flat-7.X 8/8 semantic identity and independent LFE identity passed.
- [x] Resolver matrix is frozen and consistent: 140,400 rows,
  `SUPPORTED_PASS=2,393`, `SPEC_LEGAL_NOT_IMPLEMENTED=0`.
- [x] 20-probe census count consistency passed: OpenJOC 20/20, FFmpeg 14+6,
  MediaInfo 1+19, with version-scoped claims.
- [x] Minimal CI workflow, CONTRIBUTING.md, SECURITY.md, README quick start,
  and release-path hygiene are present.
- [x] Windows x64 self-contained release workflow is checked in and refuses
  publication unless the exact source tag and pinned FFmpeg source are used.

## REVIEW

- [ ] Human license, patent, and trademark review.
- [ ] Execute Windows release CI: build FFmpeg from the pin with no GPL/nonfree
  components, generate binary and ZIP hashes, and retain the source package.
- [ ] Extract the final ZIP with system FFmpeg hidden from PATH and prove the
  bundled executable is used. Run the representative release semantic QA with
  the separately provisioned pinned OpenJOC validator.
- [ ] Publish only an explicit fresh object database containing
  `refs/heads/master`; exclude 26 desktop Codex refs and 65 unreachable local
  objects. Do not delete them without separate approval.
- [ ] Decide whether Git author email metadata is acceptable for public history.
- [ ] Provision/document the external read-only OpenJOC validator for strict and
  render validation. Configure `JOCFORGE_OPENJOC_MANIFEST` and
  `JOCFORGE_OPENJOC_EXECUTABLE`.
- [ ] Treat the full validation closure as optional until a clean-test matrix
  and local source are supplied explicitly.
- [ ] Add the legitimate JOCForge logo asset if required by the publication
  plan.

## BLOCKER

- The self-contained binary is not yet built or smoke-tested in this workspace;
  source-license closure is complete, but binary publication remains gated.
- Publication remains intentionally stopped until the REVIEW items are closed.

See [`public-release-audit.md`](public-release-audit.md) for evidence and the
machine-readable records under `artifacts/public-release-audit/`.
