# JOCForge public release audit

Audit date: 2026-09-09

Audited code state: `13494570a52cac3f3c09302df1e0799cd487799f`

License/release closure scope: this report covers the working-tree license and
release files reviewed against that implementation base. It does not assume a
commit; the final local commit hash is reported separately after verification.

This is an engineering, provenance, licensing, reproducibility, security, and
repository-hygiene audit. It is not legal advice and does not claim patent,
trademark, or copyright immunity.

## Decision

```text
JOCFORGE_PUBLIC_RELEASE_AUDIT = COMPLETE
PUBLIC_RELEASE_GATE = REVIEW_REQUIRED
READY_TO_PUBLISH = NO
```

The implementation and public smoke path are technically ready. Publication
still needs the binary evidence, human license/patent/trademark review, and an
explicit clean release export from a fresh public object database. The prior
technical/security audit has no Critical or Important findings; binary release
review remains separately gated below.

## Start and end state

The audit started at `aae23a1a2b6d64b8330c9c40b39e81b98530657b`, on branch
`master`, with a clean worktree. The requested `92d9dc0` commit existed and was
an ancestor of the starting HEAD. The intervening commits were the intended
JOCForge Flat-7.X semantic audit and fail-closed fix history.

Safe local commits made during the audit:

- `7ef5df5` — input allocation bound, path redaction, validator environment
  configuration, CI, contribution/security docs, and audit-script hygiene.
- `1349457` — public instructions for externally supplied closure inputs.

The audited code end state is `1349457…`. No push, tag, repository visibility
change, or publication was performed.

## Git and history

The complete reachable history was scanned across all local refs:

- 53 reachable commits and 903 reachable objects.
- One candidate publication ref: `refs/heads/master`.
- 26 local `refs/codex/turn-diffs/...` refs remain in the desktop workspace;
  `git rev-list --all --not master` is zero, so they add no commits to the
  candidate history. They must not be copied into a public repository.
- No tags, remotes, remote-tracking refs, notes, stash ref, alternates, replace
  refs, grafts, or reflog files exist in the audited workspace.
- `git fsck --full --no-reflogs --unreachable` reports 65 unreachable objects,
  including three old interop-script commit variants. Their trees contain only
  public JOCForge source/docs/scripts; targeted scans found no secrets,
  copyright media, or prohibited implementation material.

The existing `scripts/audit-git-release.ps1` was run after its baseline was
updated to the requested `92d9dc0`. It correctly fails on this instrumented
desktop workspace because of the 26 internal refs and 65 unreachable objects;
that is a local object-database hygiene finding, not a reachable-history
contamination result. The fresh single-branch clone had zero unreachable
objects.

History finding classifications:

- `CLEAN`: reachable code and public-spec implementation history.
- `PUBLIC_SPEC_DERIVED`: ETSI/ITU/EBU references and generic mathematics.
- `SANITIZED_HANDOFF_ALLOWED`: explicit clean-room boundary and negative
  provenance statements.
- `PRIVATE_PATH_REFERENCE_ONLY`: older progress/validator/audit paths in
  historical docs; current tracked public files were sanitized.
- `SENSITIVE_SECRET`: none found.
- `COPYRIGHT_MEDIA`: none found.
- `CONTAMINATED_PROVENANCE`: none found.
- `PROPRIETARY_IMPLEMENTATION_DERIVATION`: none found.
- `UNKNOWN_REVIEW_REQUIRED`: final human license and publication review only.

`HISTORY_REWRITE_REQUIRED = NO`. No reachable secret, prohibited proprietary
content, contaminated clean-room material, or copyrighted source media was
found. History cleanup of private path references or author metadata would
need separate authorization and is not part of this audit.

## Secrets and privacy

`gitleaks` and `trufflehog` were not installed on this host. An extended
high-confidence regex scan was run against every reachable commit and the
current tracked tree. It covered cloud-key prefixes, GitHub/OpenAI-style keys,
bearer tokens, private-key headers, connection strings, and credential-shaped
values.

Results:

- Current tracked tree secret candidates: 0.
- Reachable-history secret candidates: 0.
- Unreachable interop-variant secret candidates: 0.
- Current tracked public private-path leaks after hygiene fixes: 0.
- Current tracked media files: 0.

Older history contains a local media path in an early progress file and local
validator paths in historical reports. These are classified as
`PRIVATE_PATH_REFERENCE_ONLY`; no media bytes or credential values were found.

## Copyright and media

No `wav`, `bwf`, `ec3`, `ac3`, `mp4`, `m4a`, `caf`, `flac`, `zip`, `7z`, image,
or executable media file is tracked or reachable by filename. The controlled
fixtures are deterministic project-owned synthetic PCM. The real ADM-BWF source
used for local QA is external, is not in the repository, and is not required
for the public smoke test or CI.

The largest tracked files are explained audit tables and source scripts. The
largest is `artifacts/spec-legal-zero/final-gap-disposition.csv` at 3,337,511
bytes. Full current and reachable-history top-25 lists are in
`artifacts/public-release-audit/large-files.csv`.

## License and provenance

The root and both workspace packages declare Apache-2.0. `cargo metadata
--locked` and `cargo tree --locked --edges all` contain only the two local
packages: one direct local runtime edge and zero registry, transitive, build,
development, incompatible, or unknown dependency licenses. The full decision
and evidence are in [`license-audit.md`](license-audit.md).

The source checkout uses an external FFmpeg executable on PATH for the I0
carrier and independent decode check. The observed local 8.1 full build is
GPL-enabled and is not redistributed by this source repository. It is not the
license of JOCForge. The intended official release instead uses a separate
CI-built FFmpeg 8.1 bundle with the exact source pin and LGPL-compatible
configuration in [`../release/ffmpeg-pin.json`](../release/ffmpeg-pin.json).
The release bundle carries FFmpeg's own license files and complete source
archive; it does not merge them into JOCForge's Apache-2.0 `LICENSE`.

OpenJOC 0.16.0 remains an external validator, decoder, and renderer pinned by
source revision and executable SHA-256. The pin resolves the read-only external
manifest and executable through:

- `JOCFORGE_OPENJOC_MANIFEST`
- `JOCFORGE_OPENJOC_EXECUTABLE`

`PROVENANCE.md`, `docs/validator-provenance.md`, and `docs/legal-notes.md`
describe these boundaries. A human review remains required for final license,
patent, trademark, and any later binary-distribution interpretation.

## Reproducibility

Fresh clone was made with `--no-local --single-branch --branch master` from the
audited code state. The isolated clone passed:

- `cargo fmt --all -- --check`
- `cargo check --workspace --locked`
- `cargo clippy --workspace --all-targets --locked -- -D warnings`
- `cargo test --workspace --locked` — 173 passed across 17 suites.

The fresh clone also ran `scripts/run-mvp.ps1` with only the external pinned
OpenJOC artifact and PATH FFmpeg. It generated the synthetic fixture and
validated idx0, idx1, idx2, idx3, and idx4: 125 access units per profile,
OpenJOC inspect/render pass, FFmpeg decode pass, and deterministic re-encode.

The optional full validation closure remains external-input dependent. It now
requires `-SourceAdm` and `-CleanTestMatrixPath` (or the documented environment
variable) and fails closed when they are absent. This prevents machine-specific
directory discovery. The public smoke path does not require either private
source media or the clean-test matrix.

`NEW_USER_ONBOARDING = PARTIAL`: build/test and synthetic smoke are clear, but
the externally provisioned OpenJOC validator and optional full-closure inputs
need setup outside the repository.

## Audio semantic and capability gates

The current profile closure passed idx0 through idx4, including independent-LFE
representatives. The permanent Flat-7.X 8/8 semantic gate passed on the
post-fix `fixed-*` render corpus:

| Input | Expected output | Result |
|---|---|---|
| L | FL | PASS |
| R | FR | PASS |
| C | FC | PASS |
| Ls | Ls | PASS |
| Rs | Rs | PASS |
| Lrs | Lb | PASS |
| Rrs | Rb | PASS |
| LFE | LFE | PASS |

The earlier `identity-*` corpus recorded the pre-fix LFE→Rs failure and is
retained as historical evidence, not acceptance evidence. The fix was exercised
again by the post-fix gate and the profile/LFE closure; OpenJOC was not changed.

The resolver-generated capability matrix remains internally consistent:

- 140,400 modeled rows.
- 2,393 `SUPPORTED_PASS`.
- 2,990 `PUBLIC_AMBIGUOUS`.
- 6,240 `SEMANTIC_CEILING`.
- 10,152 `REJECTED_CAPACITY`.
- 7,800 `INVALID`.
- 110,825 `NOT_APPLICABLE`.
- 0 `RESERVED`.
- 0 `SPEC_LEGAL_NOT_IMPLEMENTED`.

No bitrate-collapsed semantic variant count is currently published; the audit
therefore does not invent the historical example value of 965.

## Interoperability census

The fresh 20-probe census used a detached OpenJOC worktree at the pinned source
revision and preserved the active OpenJOC checkout. Results:

| Family/version | Result |
|---|---|
| OpenJOC 0.16.0 | 20/20 `FULL_PASS`, 0 malformed positive AUs |
| FFmpeg 8.1 | 14/20 `FULL_PASS`; 6/20 `PARTIAL_ADDITIONAL_SUBSTREAM_UNSUPPORTED` |
| MediaInfo 21.03 | 1/20 JOC recognition; 19/20 E-AC-3-only observations |

All 20 probes were `SUPPORTED_PASS` rows. Strict/deployed, short-block,
multi-dependent, LFE, legacy-core, CMAF, and semantic-maximum interactions were
represented. Claims are version-scoped; MediaInfo is treated as a detector,
not an audio decoder or legality authority.

## Security and robustness review

The review covered malformed BW64/ADM/JSON/layout/CLI input, arithmetic bounds,
metadata allocation, path handling, process invocation, temporary files, and
output publication.

- `MAX_PCM_CHANNELS = 256` is enforced at metadata parsing, PCM frame reading,
  and pure frame decoding. Three regression tests exercise the boundary before
  allocation.
- Metadata bodies are bounded at 16 MiB; chunk offsets, padding, sample counts,
  frame offsets, and frame byte counts use checked arithmetic.
- FFmpeg and helper processes receive structured argument arrays. The census
  runner disables shell execution, redirects output, and applies timeouts.
- Output publication uses a create-new temporary file, flushes it, then
  replaces the selected target. Existing output replacement and stale-sidecar
  removal are now documented; failures remove temporary/stale artifacts.
- Representative negative UX tests returned explicit nonzero errors for missing
  input, invalid partition, insufficient capacity, malformed ADM, and a
  directory output path.

Targeted review result: `COMMAND_INJECTION_FINDINGS=0`,
`PATH_TRAVERSAL_FINDINGS=0`, `UNSAFE_OVERWRITE_FINDINGS=0` under the documented
explicit-output-path policy, `PANIC_ON_EXTERNAL_INPUT_FINDINGS=0`,
`CODE_REVIEW_CRITICAL=0`, and `CODE_REVIEW_IMPORTANT=0`.

## Public repository readiness

- README/quick start: PASS.
- Canonical Apache-2.0 root license and Cargo SPDX metadata: PASS.
- License audit and contribution terms: PASS; see `license-audit.md`.
- FFmpeg source pin and LGPL-compatible build configuration: PASS as repository
  configuration; CI-built binary hashes and extracted-bundle QA are pending.
- Self-contained Windows release workflow: PASS as a checked-in workflow; no
  release was built or published in this workspace.
- The workflow's generated `release-artifact.json` and external `SHA256SUMS.txt`
  will carry the actual FFmpeg, ffprobe, JOCForge, source-archive, and ZIP
  hashes after CI; the checked-in pin remains a source/configuration template
  with binary fields unset until then.
- Third-party notices: PASS for the planned bundle; FFmpeg remains a separate
  executable with separate license/source material.
- CONTRIBUTING clean-room policy: PASS.
- SECURITY reporting guidance: PASS; no nonexistent private mailbox is claimed.
- GitHub Actions quality workflow: PASS static audit; it runs fmt/check/clippy/
  test with read-only contents permission and installs FFmpeg on Ubuntu.
- `.gitignore`: PASS; build outputs, generated media, progress files, and local
  audit output are excluded.
- Public claims: PASS; README explicitly scopes OpenJOC/FFmpeg/MediaInfo results
  and disclaims Dolby parity, perceptual quality, certification, and legal
  conclusions.
- Brand asset: `BRAND_ASSET_PENDING`; no logo file is tracked and README does
  not reference a missing logo.

## Review items before publication

1. Run the Windows release workflow from the exact source tag. Verify the
   source pin, no-GPL/nonfree configuration, resulting binary hashes, and ZIP
   manifest.
2. Extract the ZIP in a fresh directory with system FFmpeg hidden from PATH;
   verify the bundled FFmpeg path, hashes, source package, smoke tests, and the
   separately provisioned pinned OpenJOC semantic QA.
3. Obtain human license/patent/trademark review. No legal immunity or
   third-party patent clearance is inferred.
4. Export or reconstitute the public repository from `refs/heads/master` into a
   fresh object database. Do not publish the desktop workspace's Codex refs or
   unreachable objects. This needs approval because the audit did not delete
   local objects.
5. Decide whether the author email visible in Git commit metadata is acceptable
   for publication. Changing it would require separately authorized history
   work.
6. Provision and document the external OpenJOC validator for users who want to
   run strict/render validation. The synthetic build/test path itself is clean.
7. Add the legitimate JOCForge logo asset later if branding is required.

## Required final values

```text
JOCFORGE_PUBLIC_RELEASE_AUDIT = COMPLETE
PUBLIC_RELEASE_GATE = REVIEW_REQUIRED
START_HEAD = aae23a1a2b6d64b8330c9c40b39e81b98530657b
END_HEAD = 13494570a52cac3f3c09302df1e0799cd487799f
PUSH = NOT_PERFORMED
TAG = NOT_CREATED
PUBLICATION = NOT_PERFORMED

FULL_REACHABLE_HISTORY_SCANNED = YES
HISTORY_COMMITS_SCANNED = 53
HISTORY_REWRITE_REQUIRED = NO
CONTAMINATED_HISTORY_FINDINGS = 0
PROPRIETARY_DERIVATION_FINDINGS = 0
COPYRIGHT_MEDIA_HISTORY_FINDINGS = 0
SECRET_HISTORY_FINDINGS = 0
SECRET_CURRENT_TREE = 0
PRIVATE_PATH_PUBLIC_LEAKS = 0
COPYRIGHT_MEDIA_TRACKED = 0
UNEXPLAINED_LARGE_TABLES = 0
LARGEST_TRACKED_FILE = artifacts/spec-legal-zero/final-gap-disposition.csv,3337511

CLEAN_ROOM_PROVENANCE = PASS
PUBLIC_SPEC_TRACEABILITY = PASS
PRIVATE_SANITIZER_DEPENDENCY = NO
PROPRIETARY_IMPLEMENTATION_DEPENDENCY = NO
ROOT_LICENSE = Apache-2.0
LICENSE_AUDIT = PASS
INCOMPATIBLE_DEPENDENCIES = 0
UNKNOWN_LICENSE_DEPENDENCIES = 0
THIRD_PARTY_NOTICE_REQUIRED = NO
FFMPEG_DISTRIBUTION_STATUS = external GPL-enabled FFmpeg used on PATH; not redistributed

PUBLIC_FIXTURES = PASS
COPYRIGHT_SOURCE_MEDIA_STAGED = NO
PRIVATE_REAL_MEDIA_REQUIRED_FOR_PUBLIC_TESTS = NO
FRESH_CLONE = PASS
FRESH_CLONE_BUILD = PASS
FRESH_CLONE_TEST = PASS
FRESH_CLONE_CLIPPY = PASS
README_BUILD_INSTRUCTIONS = PASS
PUBLIC_SMOKE_WITHOUT_PRIVATE_MEDIA = PASS
NEW_USER_ONBOARDING = PARTIAL

IDX0_SEMANTICS = PASS
IDX1_SEMANTICS = PASS
IDX2_SEMANTICS = PASS
IDX3_SEMANTICS = PASS
IDX4_SEMANTICS = PASS
FLAT7X_CHANNEL_IDENTITY = PASS
LFE_IDENTITY = PASS

MATRIX_ROWS = 140400
SUPPORTED_PASS = 2393
SPEC_LEGAL_NOT_IMPLEMENTED = 0
BITRATE_COLLAPSED_GENERATABLE_VARIANTS = 0
CAPABILITY_FREEZE = PASS
CENSUS_PROBES = 20
OPENJOC_RESULT = 20/20 FULL_PASS; malformed=0
FFMPEG_RESULT = 14 FULL_PASS; 6 PARTIAL_ADDITIONAL_SUBSTREAM_UNSUPPORTED
MEDIAINFO_RESULT = 1 JOC_RECOGNIZED_CORRECTLY; 19 EAC3_ONLY_FALSE_NEGATIVE
VERSION_SCOPED_CLAIMS = PASS
RESULT_COUNT_CONSISTENCY = PASS

UNTRUSTED_INPUT_REVIEW = PASS
COMMAND_INJECTION_FINDINGS = 0
PATH_TRAVERSAL_FINDINGS = 0
UNSAFE_OVERWRITE_FINDINGS = 0
PANIC_ON_EXTERNAL_INPUT_FINDINGS = 0

README = PASS
QUICK_START = PASS
CONTRIBUTING = PASS
SECURITY_DOC = PASS
GITIGNORE = PASS
CI = PASS
PUBLIC_CLAIMS = PASS
BRAND_ASSET = PENDING
CARGO_FMT = PASS
CARGO_CHECK = PASS
CARGO_CLIPPY = PASS
CARGO_TEST = PASS
CARGO_TEST_COUNT = 173
GIT_DIFF_CHECK = PASS
CODE_REVIEW_CRITICAL = 0
CODE_REVIEW_IMPORTANT = 0

TECHNICAL_RELEASE_READINESS = PASS
PROVENANCE_RELEASE_READINESS = REVIEW_REQUIRED
LICENSE_RELEASE_READINESS = REVIEW_REQUIRED
SECURITY_RELEASE_READINESS = PASS
REPRODUCIBILITY_RELEASE_READINESS = REVIEW_REQUIRED
READY_TO_PUBLISH = NO
JOCFORGE_LICENSE_CLOSURE = PARTIAL
RECOMMENDED_PROJECT_LICENSE = Apache-2.0
LICENSE_DECISION = permissive source/binary use, contributor patent grant, and no incompatible Cargo dependencies
ROOT_LICENSE = Apache-2.0
CARGO_LICENSE_METADATA = PASS
LICENSE_AUDIT = PASS
DIRECT_DEPENDENCIES = 1
TRANSITIVE_DEPENDENCIES = 0
INCOMPATIBLE_LICENSES = 0
UNKNOWN_LICENSES = 0
COPYLEFT_RUNTIME_DEPENDENCIES = NONE
FFMPEG_USAGE = EXTERNAL_EXECUTABLE
FFMPEG_BUNDLED_IN_SOURCE_RELEASE = NO
FFMPEG_SOURCE_COPIED_OR_TRANSLATED = NO
FFMPEG_LICENSE_RELEASE_BLOCKER = NO
APACHE_2_0_SUITABLE = YES
PATENT_GRANT_PRESENT = YES
THIRD_PARTY_PATENT_CLEARANCE_CLAIMED = NO
NOTICE_REQUIRED = NO
THIRD_PARTY_NOTICES_REQUIRED = YES
CONTRIBUTING_LICENSE_TERMS = PASS
CLEAN_ROOM_CONTRIBUTION_RULE = PASS
CLA_REQUIRED = NO
DCO_REQUIRED = NO
SOURCE_ONLY_FIRST_RELEASE_RECOMMENDED = NO
SELF_CONTAINED_BINARY_RELEASE_RECOMMENDED = YES
BINARY_RELEASE_MODEL = SELF_CONTAINED_BUNDLE
SOURCE_RELEASE_ALSO_PROVIDED = YES
PATENT_FREE_CLAIM = NO
DOLBY_CERTIFICATION_CLAIM = NO
LEGAL_IMMUNITY_CLAIM = NO
PUBLIC_LICENSE_WORDING = PASS
FFMPEG_PINNED = YES
FFMPEG_VERSION = 8.1
FFMPEG_COMMIT = a65b3bfe9dacc3b20597ef199d0afdd8bc8128e2
FFMPEG_SOURCE_SHA256 = b072aed6871998cce9b36e7774033105ca29e33632be5b6347f3206898e0756a
FFMPEG_CI_BUILT = NO
FFMPEG_GPL_ENABLED = NO
FFMPEG_NONFREE_ENABLED = NO
EFFECTIVE_FFMPEG_LICENSE = LGPL-2.1-or-later
FFMPEG_BUNDLED_AS_SEPARATE_EXECUTABLE = YES
LOCAL_FFMPEG_MODIFICATIONS = NONE
FFMPEG_SOURCE_AVAILABILITY = PASS
FFMPEG_BUILD_CONFIG_PUBLISHED = YES
FFMPEG_LICENSE_FILES_INCLUDED = YES
THIRD_PARTY_NOTICES = PASS
RELEASE_WORKFLOW = PASS
WINDOWS_X64_BUNDLE = NOT_BUILT
BUNDLED_FFMPEG_ACTUALLY_USED = NOT_PROVEN
SYSTEM_FFMPEG_REQUIRED = YES (source checkout only; intended bundle target is NO)
RELEASE_SMOKE = NOT_RUN
RELEASE_SEMANTIC_QA = NOT_RUN
RELEASE_ZIP_SHA256 = NOT_BUILT
RELEASE_ZIP_SIZE = NOT_BUILT
SELF_CONTAINED_BINARY_RELEASE = PARTIAL
BINARY_DISTRIBUTION_LICENSE_GATE = REVIEW_REQUIRED
REMAINING_BINARY_LICENSE_BLOCKERS = CI-built binaries, extracted smoke/semantic QA, and generated hashes are pending
NEXT_ACTION = RUN_RELEASE_CI_AND_BINARY_QA
```
