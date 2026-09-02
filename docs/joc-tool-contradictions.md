# JOC tool contradiction log

> Verified: 2026-09-02
> No proprietary media, private matrices, or contaminated workspace were
> used. The screenshots were treated as user-observed tool output, not as a
> serialization source.

## Normative ETSI facts

- ETSI TS 103 420 Table 32/33 defines JOC downmix configuration 0 as 5.X
  (`L,R,C,(LFE),Ls,Rs`) with five JOC matrix input channels, and says that an
  LFE, when present, is bypassed rather than processed by the JOC decoder.
- ETSI TS 103 420 Table 40 assigns payload ID 11 to OAMD and payload ID 14 to
  JOC. Table 41 defines the payload configuration, including `codecdatae=1`.
- ETSI TS 102 366 Annex H permits an EMDF container in reserved E-AC-3
  locations including `addbsi`, `auxdata`, and `skipfld`.

Sources:

- https://www.etsi.org/deliver/etsi_ts/103400_103499/103420/01.01.01_60/ts_103420v010101p.pdf
- https://www.etsi.org/deliver/etsi_ts/102300_102399/102366/01.03.01_60/ts_102366v010301p.pdf

## Observed tool results

For the current JOCForge outputs, OpenJOC parses EMDF/JOC and reports
`normative_compliant`, with zero malformed EMDF attempts. MediaInfo v26.01,
as shown by the user, reports only `E-AC-3` and does not set `JOC`.

MediaArea's public `File_Ac3.cpp` shows that its EMDF pre-detector rejects an
EMDF candidate when `codecdatae` is nonzero, while the same parser's JOC
reporting path requires a successfully populated `joc_num_objects_map` and
JOC addbsi state. This creates a tool-recognition condition that is narrower
than the ETSI Table 41 value used by this clean writer.

Source: https://github.com/MediaArea/MediaInfoLib/blob/master/Source/MediaInfo/Audio/File_Ac3.cpp

## Closure interpretation

This is a genuine standards/tool contradiction, not evidence that a text label
can be added to the stream. The clean result is therefore recorded as:

`ETSI_EMDF_JOC = PRESENT_BY_PUBLIC_SYNTAX`

`OPENJOC_JOC = PARSED_AND_STRICT_ACCEPTED`

`MEDIAINFO_26_01_JOC = NOT_RECOGNIZED`

`MEDIAINFO_INTEROPERABILITY = OPEN`

The project must not change `codecdatae` or invent an addbsi value solely to
make MediaInfo display `E-AC-3 JOC` without a new ETSI-grounded test proving
the resulting stream remains conforming.
