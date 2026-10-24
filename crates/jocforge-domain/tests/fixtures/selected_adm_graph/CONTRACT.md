# Selected ADM graph and physical-host identity

Status: `IMPLEMENTATION_READY=NO_FOR_SELECTED_CHAIN; CANDIDATE_VALID_METADATA_PARSER_ONLY`.
Delivery scope: internal implementation only. The package contains AXML,
CHNA, and DBMD metadata chunks from the candidate `hanabi.wav`, but no WAV
data chunk, PCM, audio digest, DLL, disassembly, key, or authentication material.

This supplement validates a source-local F01G parser and identity slice for
the metadata chunks included here. Its binding to the main selected job and
the 4,247-source reference encode has not been established, so the graph rows
must not yet be joined to the implementation's selected event vectors or
used to claim the product path is connected.

## Source identity blocker

The metadata receipt identifies `hanabi.wav` with 13,888,071 source frames.
The current F01E event/control vectors declare `N=6,522,906` and 4,247 normal
callbacks. No evidence yet proves these are the same WAVE with a selected
trim/range, or different source fixtures. Treat this package as a candidate
valid-metadata parser input until Wash supplies a hash-bound source/job/range
join. Matching track counts, IDs, or row values is not enough to infer that
identity.

## Input and graph contract

`source_metadata.xml` is the selected source AXML chunk; `source_track_associations.bin`
is its CHNA chunk. The source metadata receipt binds them to the selected
48 kHz, 38-channel, 24-bit WAV. Only these metadata chunks were read; the data
chunk was not read or hashed.

The selected CHNA chunk contains a 4-byte header and 38 fixed 40-byte entries.
Each entry has a one-based physical `track_index`, an `ATU_*` track UID, an
audioTrackFormat ID, and an audioPackFormat ID. The selected host index is
`track_index - 1`, yielding the physical host domain 0..37.

For the selected valid input, resolve the exact AXML references:

```text
audioTrackUID -> audioTrackFormat -> audioStreamFormat
              -> audioChannelFormat
audioTrackUID -> audioPackFormat
audioTrackUID -> the single referencing audioObject
```

The resulting graph has 38 rows: 10 DirectSpeakers bed tracks and 28 object
tracks. Bed identity comes from the first selected block's speaker label and
the selected bed-role mapping. The direct LFE track is physical host 3.
`REAL-GRAPH-TRACK-IDENTITIES.json` is the hash-bound expected graph for these
actual input chunks; its 38 rows were independently recomputed from the AXML
and CHNA bytes.

The same source's `source_metadata.dbmd` is the actual 580-byte DBMD metadata
chunk, bound by offset and SHA-256 in `DBMD-SOURCE-RECEIPT.json`. It contains
sections 7, 9, and 10 with valid checksums. `DBMD-PARSER-EXPECTED.json` gives
the frozen selected parser's four authored fields and 38 track records. The
track records are indexed by zero-based `track_index`; join them to graph rows
by the physical host derived from CHNA, not by object ID or object ordinal.
The independent parse computes raw flags/packed bytes and then the selected
enum48/enum52/flag64 fields. It does not claim general malformed DBMD behavior
is native-qualified.

## Native descriptor and source identity

The native lookup key is the `ATU_*` track UID. It is not an
`AC_*` audioChannelFormat ID. Across four independently scoped native loads,
152 UID lookup calls bind the graph UID to bed/object descriptor construction;
40 bed and 112 object descriptor stores bind each selected semantic key and
physical host. A repeated lookup of the same UID returns the same one-based
track index; this is lookup behavior, not duplicate graph insertion behavior.

There are 37 regular physical hosts: all hosts 0..37 except direct LFE host 3.
For this selected input they are in ascending physical order. The observed
regular global source index is `11 + rank_in_regular_hosts`, producing
indices 11..47. The source count is 48: an 11-entry virtual prefix plus 37
regular rows.

The selected 11-entry virtual prefix geometry is also supplied by the
hash-bound prefix constructor vectors. Its Q15 recipe uses endpoint 32767,
midpoint 16384, and denominator 32768 to construct the selected eleven-point
layout. The native constructor's 33 XYZ words and 143 defined metadata words
match the independently regenerated values. The constructed table is the
same owner table consumed by eight main routing/consumer calls; 264 runtime
XYZ words and 440 attribute components match. Attribute components
`[0, 0, 0, 3, 2]` pack with shifts `[0, 1, 4, 8, 10]` to 2816. Renderer window
3072 and host frame 1536 are separate domains.

This closes only the selected prefix geometry and its same-table consumer
identity within its own native capture. Its same-job relation to the candidate
AXML/CHNA file and the F01E vectors is not established. Source field 0 is not
reconstructed, and skip at field 4 is observed without a complete runtime
transition rule. Seven earlier measurement-side consumer calls were not
bound to this constructor and remain outside the receipt.

The 296 source-field boundary cases bind each regular row's 13 defined U32
metadata words from its input control52 words. They are downstream field
conversion vectors, not an independent producer of AXML coordinates or
source XYZ/attributes. The native object source copies those 13 control52
words into its defined fields; the bed source uses the selected bed-role
literal XYZ and defaults the other defined values as shown in the vectors.

## Identifier behavior and model boundary

The selected canonical object IDs such as `AO_1001` map to the semantic key
used by the graph. Native formatter probes also accept forms that the frozen
Python model's strict four-hex policy rejects, including a trailing suffix,
trailing whitespace, or whitespace after the `AO_` prefix. Hex letters are
case-insensitive. The selected source uses canonical IDs, so this does not
change its graph. Preserve the distinction: the model policy is a valid-input
contract; the native formatter's malformed-input behavior is separately
observed in `OBJECT-ID-NATIVE-OBSERVATIONS.json`.

An accepted or rejected formatter call consumes its owned input string. A
rejected call leaves its output key sentinel unchanged; that does not mean
the entire operation made no writes. Existing UID lookup behaves differently:
missing and empty queries return 0, leave the output sentinel unchanged,
and preserve the query string. The full XML loader's duplicate/missing
reference behavior remains untested.

## Limits that remain open

- The selected prefix geometry and selected attributes are recovered, but
  source field 0 and the complete skip-at-field-4 transition remain open.
- Candidate `hanabi.wav` source binding to the main selected job and its
  4,247-callback vectors remains unresolved.
- Earlier measurement-side prefix consumers were not joined to this native
  constructor and must not be merged on value equality alone.
- Full AXML numeric production into every source field and complete control38
  generation are not covered.
- The selected input's CHNA order is ascending by physical host. Arbitrary
  reordering or interleaving is not qualified.
- Native malformed/duplicate XML graph policy and automatic reset/rollback
  are not observed.
- General malformed DBMD policy is not native-qualified by the successful
  selected chunk parse.
- This is not a full 48-source numeric installed-state receipt, a complete
  F01G closeout, or an end-to-end ADM-to-E-AC-3 JOC run.

