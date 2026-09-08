# OpenJOC, FFmpeg and MediaInfo interoperability census

This report publishes a version-scoped result from a deterministic 20-stream
positive corpus. The census evidence was generated against JOCForge source head
`8919c8c75beb3a795539fd7ddb00ad4ce888b00a` and OpenJOC source head
`a52c35d21e68e18dbaa8fa18accfbfea44087367`. The publication snapshot is
JOCForge `aa6ad11be84cd726573780b48b0db90b6f1c3446`; no Rust production source
changed between the tested source and that publication commit.

## Executive summary

A 20-stream corpus was selected from JOCForge configurations already classified
`SUPPORTED_PASS` under the project's public ETSI capability model.

- OpenJOC fully parsed all 20 probes: 19 raw probes rendered and the CMAF probe
  passed its container-decode path.
- FFmpeg 8.1 fully decoded 14 and partially handled 6 because the tested
  additional E-AC-3 substreams were not fully assembled/decoded by that build.
- MediaInfo 21.03 identified 1 probe as JOC and labeled 19 as E-AC-3 only.

These are version-scoped interoperability observations, not claims about every
decoder or detector in existence.

## Scope and implementation families

The JOCForge composition matrix is the public legality authority for probe
selection. Only rows classified `SUPPORTED_PASS` entered the positive corpus.
The three counted implementation families are:

- OpenJOC — an independent specialist JOC parser, decoder and renderer.
- FFmpeg — a mainstream open-source E-AC-3 decoder implementation family.
- MediaInfo — a mainstream media metadata and format-detection family.

MediaInfo is not an audio decoder and its labels are not legality evidence. LAV
may use FFmpeg decoding infrastructure and is not counted as an independent
decoder family. PotPlayer file-information output may reflect MediaInfo-family
detection and is not counted as an independent detector family. Wrappers and
frontends do not add votes.

## Tool versions

| Tool | Tested version | SHA-256 | Local path policy |
|---|---|---|---|
| OpenJOC | OpenJOC 0.16.0 | `c740767304775648e18e3fdf8e490744e3c9521e27a8e2fca2aad9027b12977c` | external checkout path omitted |
| FFmpeg | 8.1-full_build-www.gyan.dev | `865b4a599b589982cbfa875c32eec92abbe0d6e78120ab3d28701b59e95d8265` | executable path omitted |
| ffprobe | 8.1-full_build-www.gyan.dev | `d1c3740b2c2a23e3b8990afdf08492a9879fe1845ccdbc43e5cb2404ac01b2f1` | executable path omitted |
| MediaInfo | MediaInfoLib 21.03 | `62bed8fd20d54e862c35f3a220022da8074538e5ead67895c4c5cbb03100a2a8` | executable path omitted |

The canonical machine-readable tool record is
[`interop-census/tool-versions.json`](interop-census/tool-versions.json).

The census OpenJOC pin is intentionally independent of the frozen validator
identity in `validation/openjoc-validator.json`. Existing closure scripts use
that frozen validator for their separate acceptance loop; this census uses the
tested OpenJOC release source head and executable SHA listed above. The two
identities must not be conflated, and this report is not frozen-validator
evidence.

The two clean reruns produced the same hashes for the raw normalized records.
The path-bearing records differ from the canonical publication copies below;
`summary.json` has no path-bearing fields and consequently retains the same hash.

| Raw record | SHA-256 |
|---|---|
| `corpus-manifest.json` | `80e42fd310d3cc16eddb36150e8db49e3898b5462bcb37b3ee185d7cdf4899bf` |
| `detailed-matrix.csv` | `5255146c08f9f93d48085a4567d5566572fb4d720b0ec6dfa191e72758486c6d` |
| `detailed-matrix.json` | `2dc3b623b8e84e28e1b14ec422cc51f534221ebff2969c7de9ad9172494f830c` |
| `summary.json` | `79ae3ae41a190e94ce21ec95bc1c8695b53c00485f09cf7580e1d55235def8b9` |
| `tool-versions.json` | `2b0bbc4ae49df1d31e018af411d1a42e81f7f7ab8fde4e44cadabe57b6a589f4` |

The staged canonical publication records have these hashes:

| Canonical record | SHA-256 |
|---|---|
| `interop-census/corpus-manifest.json` | `6b72c9acea460f80404aa04ba14897fb54eab4155977d2844acbf18c9979dbfd` |
| `interop-census/detailed-matrix.csv` | `29d9604d5296ae1280c40159319a1d8092194c47bd11e836f7fe26cc82eadb89` |
| `interop-census/detailed-matrix.json` | `bc5a51e375042979ed9a13de67e0134bfcca14b24bcdb31704c8aae09484474b` |
| `interop-census/summary.json` | `79ae3ae41a190e94ce21ec95bc1c8695b53c00485f09cf7580e1d55235def8b9` |
| `interop-census/tool-versions.json` | `8d572bd05fc03a209c26f8701722bdd53d5669c8e4bfa90f825a80c168c37ad0` |

## Corpus methodology

The runner regenerates the capability matrix, creates project-owned synthetic
fixtures, and selects representatives across profile, carriage, signaling,
block partition, dependent topology, LFE ownership, bitrate and container. The
20 probes cover baseline JOC, idx0 through idx4, strict and deployed signaling,
I0 skip-field carriage, all three normal short-block partitions, independent
and dependent LFE, D1+ and higher multi-dependent topology, legacy AC-3 core,
CMAF, an orthogonal composition, and the semantic maximum.

OpenJOC is scored by inspection, malformed-AU count, render health, and CMAF
container decode. FFmpeg is scored by base decode and tested additional-
substream behavior. MediaInfo is scored only by recognition fields. The compact
corpus has no negative controls and requires no copyrighted source media.

## Legal-positive probe selection

The complete modeled matrix contains 140,400 rows. Its current counts are:

| Matrix verdict | Cells |
|---|---:|
| `SUPPORTED_PASS` | 2,393 |
| `PUBLIC_AMBIGUOUS` | 2,990 |
| `NOT_APPLICABLE` | 110,825 |
| `SEMANTIC_CEILING` | 6,240 |
| `REJECTED_CAPACITY` | 10,152 |
| `INVALID` | 7,800 |
| `RESERVED` | 0 |
| `SPEC_LEGAL_NOT_IMPLEMENTED` | 0 |

Every public positive probe in the manifest has
`expected_public_classification: SUPPORTED_PASS`.

## Summary results

| Implementation family | Result |
|---|---:|
| OpenJOC | 20 / 20 `FULL_PASS` (19 raw renders + 1 CMAF container decode); positive malformed AUs: 0 |
| FFmpeg 8.1 | 14 / 20 `FULL_PASS`; 6 / 20 `PARTIAL_ADDITIONAL_SUBSTREAM_UNSUPPORTED` |
| MediaInfo 21.03 | 1 / 20 `JOC_RECOGNIZED_CORRECTLY`; 19 / 20 `EAC3_ONLY_FALSE_NEGATIVE` |

The six FFmpeg partial results are not malformed, invalid, or broken JOCForge
output. They are the tested build's additional-substream coverage boundary.
`EAC3_ONLY_FALSE_NEGATIVE` means MediaInfo reported E-AC-3 without identifying
the probe's JOC semantics; it does not mean that MediaInfo attempted decoding.

## Probe-by-probe table

`FULL`, `PARTIAL`, `JOC`, and `EAC3 ONLY` are concise presentation cells. The
machine-readable records retain the exact status names.

| Probe | Key feature | OpenJOC | FFmpeg 8.1 | MediaInfo 21.03 |
|---|---|---|---|---|
| `p01-baseline-idx0-strict-6` | ordinary 5.X JOC baseline | FULL | FULL | EAC3 ONLY |
| `p02-baseline-idx0-deployed-6` | deployed EMDF pair | FULL | FULL | EAC3 ONLY |
| `p03a-idx0-skipfld-strict-6` | strict I0-only skip-field | FULL | FULL | EAC3 ONLY |
| `p03-idx0-skipfld-deployed-6` | deployed I0-only skip-field | FULL | FULL | JOC |
| `p04-idx1-flat-7x-strict-6` | idx1 Flat-7.X | FULL | FULL | EAC3 ONLY |
| `p05-idx2-5x2-strict-6` | idx2 5.X+2 | FULL | FULL | EAC3 ONLY |
| `p06-idx3-phase-strict-6` | idx3 phase signaling | FULL | FULL | EAC3 ONLY |
| `p07-idx4-phase-5x2-strict-6` | idx4 phase signaling | FULL | FULL | EAC3 ONLY |
| `p08-idx2-short-3x3` | normal 3+3 short blocks | FULL | FULL | EAC3 ONLY |
| `p09-idx2-short-2x2x2` | normal 2+2+2 short blocks | FULL | FULL | EAC3 ONLY |
| `p10-idx2-short-1x6` | normal 1+1+1+1+1+1 short blocks | FULL | FULL | EAC3 ONLY |
| `p11-idx0-short-3x3-independent-lfe` | independent LFE outside JOC matrix | FULL | FULL | EAC3 ONLY |
| `p12-idx2-dependent-lfe-supplement` | dependent LFE supplement | FULL | PARTIAL | EAC3 ONLY |
| `p13-idx2-dependent-lfe-replacement` | dependent LFE replacement | FULL | PARTIAL | EAC3 ONLY |
| `p14-idx1-d0-d1-strict-6` | D0+D1 dependent topology | FULL | PARTIAL | EAC3 ONLY |
| `p15-idx2-d0-d2-strict-6` | D0..D2 dependent topology | FULL | PARTIAL | EAC3 ONLY |
| `p16-idx2-d0-d1-short-deployed` | orthogonal short/deployed composition | FULL | PARTIAL | EAC3 ONLY |
| `p17-idx1-legacy-ac3-strict` | AC-3 core plus E-AC-3 dependent | FULL | FULL | EAC3 ONLY |
| `p18-idx0-cmaf-ec3-dec3` | CMAF ec-3/dec3 six-block JOC | FULL | FULL | EAC3 ONLY |
| `p19-idx4-d0-d5-short-monster` | semantic maximum idx4 D0..D5 3+3 | FULL | PARTIAL | EAC3 ONLY |

## Key compatibility boundaries

OpenJOC fully parsed all five tested profile families, all three normal
short-block partitions, both LFE ownership variants, multi-dependent topologies,
legacy-core carriage, CMAF, and the semantic maximum. The raw probes passed the
render gate; the CMAF probe passed its container-decode gate. This is the
specialist result for OpenJOC 0.16.0 in this corpus, not a universal superiority
claim.

FFmpeg 8.1 fully decoded the single-dependent representatives and the tested
six-block/short-block cases. The six partial cases are exactly the dependent-LFE
and D1+ or higher substream probes: `p12`, `p13`, `p14`, `p15`, `p16`, and `p19`.

MediaInfo 21.03 recognized JOC only for the deployed I0-only skip-field probe
`p03`. The other 19 probes were reported as E-AC-3 only. This is detector
behavior for this version, not an audio-decoding result.

## Controlled-pair findings

### Strict versus deployed

The raw strict/deployed pair `p01`/`p02` held profile, ETSI raw carriage,
six-block grouping, dependent count, LFE state and bitrate constant. OpenJOC and
FFmpeg were FULL for both; MediaInfo labeled both E-AC-3 only. The I0-only
skip-field pair `p03a`/`p03` held the same structural role while changing the
signaling profile: strict was E-AC-3 only, while deployed was labeled JOC by
MediaInfo. That exact observation must not be generalized into a standards
requirement.

### Six blocks versus short blocks

`p08`, `p09`, and `p10` kept the raw single-dependent composition constant while
testing `3+3`, `2+2+2`, and `1+1+1+1+1+1`. OpenJOC and FFmpeg were FULL for all
three; MediaInfo labeled all three E-AC-3 only. Short-block handling is a
separate tested boundary from general JOC recognition.

### D0 versus D1+

The simpler single-dependent path was FULL in FFmpeg for the tested cases. The
tested additional-substream cases were PARTIAL_ADDITIONAL_SUBSTREAM_UNSUPPORTED.
This states only what the tested FFmpeg 8.1 build demonstrates; FFmpeg behavior
does not define JOC legality.

### Profile findings

idx0, idx1 Flat-7.X, idx2 5.X+2, idx3 phase signaling, and idx4 phase signaling
all had FULL OpenJOC inspection/render evidence. The phase profiles are
signaling variants in the JOCForge model; no extra PCM phase transform is claimed.
MediaInfo labels do not establish profile support.

## Semantic-Maximum Raw JOC Probe

`p19-idx4-d0-d5-short-monster` is the current generator-compatible semantic
maximum: idx4, D0..D5, shared `3+3`, ETSI strict signaling, and 4224 kbps. Its
matrix row is `SUPPORTED_PASS`. OpenJOC was FULL, FFmpeg was PARTIAL due to the
additional-substream boundary, and MediaInfo reported E-AC-3 only.

## Reproduction

From the JOCForge repository root:

```powershell
.\scripts\run-interop-census.ps1 -OpenJocRoot ..\OpenJOC
```

Prerequisites are Rust/Cargo, an OpenJOC Git checkout containing source commit
`a52c35d21e68e18dbaa8fa18accfbfea44087367`, FFmpeg 8.1 with ffprobe, and
MediaInfo 21.03. Supply the active OpenJOC checkout with
`-OpenJocRoot` or `JOCFORGE_OPENJOC_ROOT`; the runner creates or reuses a clean
detached worktree at the exact census pin, builds the CLI there, and does not
require the active checkout to be at that commit or clean. Use
`-OpenJocWorktreeRoot` for a reusable worktree path and
`-KeepOpenJocWorktree` to retain a worktree created by the run. No
machine-specific path is required. Pass
`-OutputRoot artifacts\interop-census-rerun` for an independent clean rerun.

The historical publication records the tested executable SHA-256
`c740767304775648e18e3fdf8e490744e3c9521e27a8e2fca2aad9027b12977c`. A fresh
source build records its own binary SHA-256; native Rust executable bytes are
not assumed to be identical across build environments. The authoritative
reproduction identity is the exact detached OpenJOC commit, version, and
source-built executable path, while the historical SHA remains visible as
publication evidence.

The runner generates `corpus-manifest.json`, `detailed-matrix.csv`,
`detailed-matrix.json`, `summary.json`, `tool-versions.json`, and one probe
directory per positive case under the selected output root. The canonical
normalized records are kept under [`docs/interop-census/`](interop-census/).
The measured clean reruns produced identical normalized hashes; this is a
statement about the measured reruns, not a universal bit-for-bit determinism
claim for every tool build.

## Limitations and claim scope

The corpus uses deterministic synthetic audio. OpenJOC render health checks
finite PCM and positive sample count; they do not claim perceptual parity.
FFmpeg output is an interoperability observation and does not decide ETSI
legality. MediaInfo is not a decoder. The modeled public capability claim is
limited to 140,400 JOCForge cells, of which 2,393 are `SUPPORTED_PASS` and 0
are `SPEC_LEGAL_NOT_IMPLEMENTED`.

This publication does not claim every imaginable Dolby Atmos stream, complete
Dolby compatibility, proprietary encoder parity, byte-equivalence with Dolby
encoders, patent clearance, trademark authorization, certification, or legal
immunity. JOCForge's clean-room framing describes implementation provenance,
not a legal conclusion.

## Release-facing claim set

Approved release-note concepts are: independent clean-room Rust E-AC-3 JOC
decoder/renderer; standards-aware JOC stream inspector; the modeled public ETSI
capability space; 2,393 `SUPPORTED_PASS` combinations with zero modeled
`SPEC_LEGAL_NOT_IMPLEMENTED` cells; OpenJOC 20/20 in this corpus; FFmpeg 8.1
14 FULL plus 6 PARTIAL; and MediaInfo 21.03 1/20 JOC recognition.

Avoid claims of 100% Dolby Atmos compatibility, official Dolby compatibility,
bit-perfect Dolby decoding, universal support, patent-free status, or that
another decoder is broken.

## QA and historical transparency

The first complete workspace test run contained one transient failure. A
targeted rerun passed, the subsequent full suite rerun passed, and the clean
census reruns were reproducible. The detail is recorded in
[`docs/interop-census-qa.md`](interop-census-qa.md) without presenting the
transient as a current failure.

No production Rust source changed for this publication task.
