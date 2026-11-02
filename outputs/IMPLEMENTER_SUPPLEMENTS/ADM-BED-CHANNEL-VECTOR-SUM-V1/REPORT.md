# 人是猫 ADM bed-channel vector count

The authorized, hash-bound AXML/CHNA metadata receipt resolves ten
DirectSpeakers bed tracks. Each bed row has exactly one referenced AXML
channel ID, so the ten metadata vector lengths are all 1 and sum to 10. The
same metadata has 28 object rows and 38 total CHNA tracks.

Combined with the static DEE storage formula
`object_source_count + sum(bed_row_channel_vector_lengths)`, the best-supported
metadata-derived candidate master count is `28 + 10 = 38`. This removes the
previous assumption about the parsed metadata row vector itself; it does not
prove the native storage interface's runtime row identity or the selected
instance's `master_descriptor+0x0A` value. No BWF/PCM was opened here, and no
DEE run occurred.
