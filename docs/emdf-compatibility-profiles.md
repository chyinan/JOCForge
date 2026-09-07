# EMDF compatibility profiles

Verified: 2026-09-07

JOCForge keeps E-AC-3/JOC carriage placement separate from EMDF signaling. A
carriage selects where the metadata travels; an EMDF profile selects the
payload configuration used by the single semantic writer.

## Supported states

| State | CLI selection | Carriage | EMDF `codecdatae` |
| --- | --- | --- | --- |
| ETSI Strict | `--compat etsi` (default) | I0 plus dependent D0, EMDF in D0 `auxdata` | 1 for IDs 11 and 14 |
| I0 Skip-Field with strict EMDF | `--carriage i0-skipfld --emdf-profile etsi` | I0-only, EMDF in audio-block `skipfld` | 1 for IDs 11 and 14 |
| Deployed Compatibility | `--compat deployed` | I0-only, EMDF in audio-block `skipfld` | 0 for IDs 11 and 14 |

Advanced flags may select the two dimensions explicitly. The deployed preset
is initially limited to `idx0` at 1024 kbps because that is the validated
I0-only frame budget. The default remains standards-strict and never silently
changes to the deployed profile.

## Deployed compatibility boundary

The deployed profile is intentionally narrow. It keeps JOCForge's current
values for group ID, payload order, frame alignment, duplicate/priority/
processing controls, downmix index, addbsi, and payload IDs. It does not add
payload IDs 1 or 2, copy DEE payload bytes, or change the audio, reduction,
temporal, matrix, OAMD, or object-assignment algorithms.

`codecdatae=0` differs from the published ETSI TS 103 420 strict profile. It is
classified as an observed deployed compatibility deviation, based on the
audited external stream and the measured MediaInfo behavior. It is not claimed
as a normative ETSI alternative or as universal encoder consensus.

The MediaInfo v26.05 detector currently rejects an EMDF candidate when it sees
`codecdatae=1`; it then cannot reach its JOC object-count heuristic. The
deployed profile uses `codecdatae=0` and may therefore improve MediaInfo
recognition. MediaInfo's label is an interoperability observation, not
JOCForge's normative acceptance authority. OpenJOC strict validation remains
unchanged; deployed output is validated with the explicit compatibility
profile where appropriate.

The pinned OpenJOC renderer accepts the two-field deployed deviation for all
9,042 AUs. Its 2.0 common-profile stereo shortcut is reserved for an exact
historical seven-deviation binding, so a two-field deployed stream is rendered
through the normal JOC reconstruction path in that layout. This can make a
2.0 rendered PCM comparison differ even when the FFmpeg core PCM and all JOC/
OAMD payload bodies are identical. In a 5.1 spatial layout, strict and
deployed renders are bit-identical; this distinction is validator behavior,
not a JOCForge audio or matrix change.

## Provenance

- Carriage topology and EMDF syntax are derived from public ETSI
  specifications.
- The orthogonal carriage/profile API is a JOCForge architecture decision.
- `codecdatae=0` is an observed deployed compatibility choice, not a
  proprietary algorithm or a strict-profile requirement.
