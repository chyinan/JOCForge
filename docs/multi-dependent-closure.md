# Multi-dependent raw JOC closure

This is an advanced raw-elementary conformance boundary, not a normal encoder
preset. ETSI TS 102 366 V1.4.1 permits up to eight sequential dependent
substreams per independent substream (`D0..D7`). ETSI TS 103 420 V1.2.1
requires all associated dependents to reach JOC and places the OAMD/JOC EMDF
in the last dependent.

JOCForge's canonical source-derived writer supports validated no-LFE Table-47
raw profiles through `I0+D0..D4` for idx0/idx3 and `I0+D0..D5` for idx1/idx2/idx4.
It also supports one source-derived dependent-LFE owner in D0 at every useful
count, both as a supplement when I0 has no LFE and as a replacement when I0
owns LFE. It emits actual source-derived audio for every dependent,
rejects duplicate full-band ownership and multiple LFE owners, and places the
sole JOC/OAMD carrier on the final dependent. The pinned OpenJOC validator
checks strict parse and full programme render with one LFE outside the JOC
matrix.

The public raw count maximum remains eight. `D0..D6` and `D0..D7` are recorded
as `SEMANTIC_CEILING` for the current canonical JOC plan: no additional
non-empty, non-duplicated dependent role exists after the Table-47 full-band
groups are consumed, and a dependent LFE must share a full-band dependent
rather than fill an LFE-only slot. This is distinct from `REJECTED_CAPACITY`.
CMAF remains independently restricted to one `I0` and optional `D0` by TS
103 420 Annex E.

FFmpeg is not a normative legality gate. Its public decoder currently decodes
substream ID 0 and explicitly skips additional nonzero substream IDs. Thus
`I0+D0` may report FFmpeg `PASS`, while a structurally valid
`I0+D0+D1+…` cell reports `UNSUPPORTED_DECODER_FEATURE` after OpenJOC
strict/full-render success. Dependent-LFE count-1 cells remain FFmpeg PASS in
the generated evidence; additional dependent cells are interoperability-only
unsupported. See [the detailed interoperability boundary](ffmpeg-interoperability-boundary.md)
and the [public semantics matrix](multi-dependent-public-semantics.md).

The closure emits `artifacts/dependent-channel-role-census.csv`, per-fixture
`*.ownership.json` reports, and `artifacts/multi-dependent-closure/channel-ownership.csv`.
Each ownership report includes the final JOC owner, LFE owner, map roles, and
source perturbation result.
