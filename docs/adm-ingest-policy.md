# ADM/BW64 ingest policy

## DirectSpeakers LFE authority

`LFE_SEMANTIC_AUTHORITY` is the public ADM DirectSpeakers channel metadata:

1. `audioChannelFormat/audioBlockFormat/speakerLabel`, matched using the
   public BS.2051/common-definition label rules and the public ETSI custom
   DirectSpeakers labels; `LFE`, `LFEL`, `LFE1`, and `RC_LFE` map to LFE1,
   while `LFER`, `LFE2`, and `RC_LFE2` map to LFE2.
2. `audioChannelFormat/frequency/lowPass`, when present and finite, also
   establishes the LFE path.

These fields are semantic metadata. `audioObjectName`,
`audioChannelFormatName`, filenames, source order, and arbitrary labels are
never used to classify LFE. A DirectSpeakers channel without either semantic
field, or with a non-LFE speaker label that conflicts with `lowPass`, is
rejected before PCM routing. This is the fail-closed policy for the narrow
static ingest subset.

The public references for the LFE rule are [EBU Tech 3388, sections 6.3 and
8.1](https://tech.ebu.ch/files/live/sites/tech/files/shared/tech/tech3388v1.pdf)
and [ETSI TS 103 420, tables B.9 and B.10](https://www.etsi.org/deliver/etsi_ts/103400_103499/103420/01.02.01_60/ts_103420v010201p.pdf).

## CHNA duplicate policy

EBU Tech 3364 describes the CHNA relationship and identifies
`audioTrackUID` as the unique identifier for a track. JOCForge's accepted
static full-track subset additionally requires one CHNA row per physical PCM
track: `track_index` and `audioTrackUID` are each unique, non-zero, in range,
and each row has non-empty track and pack references. Any duplicate or
conflicting relation is rejected. Exact duplicate rows are rejected as
malformed input rather than normalized, because this subset has no temporal
TrackUID-definition model.

No first-write-wins or last-write-wins behavior exists. The CHNA pack
relationship is checked against each referenced ADM object pack before source
expansion, and duplicate ADM TrackUID references are rejected as well.

The public references are [EBU Tech 3364, section 3.1](https://tech.ebu.ch/files/live/sites/tech/files/shared/tech/tech3364v1_0.pdf)
and [EBU Tech 3285 Supplement 7](https://tech.ebu.ch/publications/tech3285s7).

## LFE carrier headroom

All LFE channels are summed once into the core LFE carrier and passed through
the same normalized `0.90` absolute-peak policy used for source-derived
carriers. The metrics sidecar records the measured post-policy peak and
remaining headroom. This is a bounded structural invariant, not a perceptual
loudness claim.
