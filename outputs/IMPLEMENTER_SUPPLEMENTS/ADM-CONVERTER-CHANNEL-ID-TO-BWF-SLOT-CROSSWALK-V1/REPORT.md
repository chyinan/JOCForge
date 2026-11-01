# ADM converter channel ID to BWF slot crosswalk

## Result

For the authorized 38-channel “人是猫” ADM, static converter metadata and a hash-bound CHNA crosswalk compose to a per-channel mapping. CHNA track index is 1-based; the interleaved BWF slot is `track_index - 1`. The converter stores a separate `ConverterChannelInputId` at the descriptor slot keyed by the same track-index formula.

The logical converter ID happens to equal the BWF slot on all 38 rows of this sample. This is sample-specific ordering, not an identity rule. At the bed/object boundary, bed track 10 (`AO_1001`, bed ordinal 9) maps to logical ID 9 / BWF slot 9; object track 11 (`AO_100B`, `BACKVo-L`) maps to logical ID 10 / BWF slot 10. The distinguishing tuple includes pack type, AudioObjectID, bed ordinal, CHNA UID, and track index.

## Read and image scope

The metadata-only extractor revalidated `fmt `, AXML, and CHNA chunk hashes. It sought over the 743,611,284-byte `data` chunk; zero PCM data bytes were read. The whole-file SHA is inherited from an earlier receipt and was not recomputed in this pass.

The static mapping is tied to converter PE SHA-256 `9972be4f952ca5f9121fa83ef941b29d4c1e6baf0ff1f708baaaedffa1bd76a8`. Run149 did not establish the mapped module SHA. The current E build with SHA `bdb502fe…e3cc8d80` was not loaded in that run; do not promote the static crosswalk to a runtime image claim.

Actual native row selection, cursor/EOF, PCM row consumption, and the meter descriptor join remain `NOT_OBSERVED` or `UNBOUND`. This package contains no ADM metadata table, converter source, binary, PCM, or payload.
