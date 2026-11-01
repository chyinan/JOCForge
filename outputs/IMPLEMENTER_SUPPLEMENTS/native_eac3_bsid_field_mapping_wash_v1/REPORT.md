# Wash review: E-AC-3 BSID and DialNorm field crosswalk

This package resolves the semantic label of the R141 same-parent record member at relative offset +8. Ghidra shows the serializer writes the sync word and then the E-AC-3 header field widths; the +8 member is the first five-bit field after the three-bit and one-bit fields. The next five-bit field is at relative offset +32.

The public FFmpeg E-AC-3 parser skips five bits for BSID and then reads five bits for DialNorm. This matches the serializer's field order: +8 is BSID; +32 is DialNorm. The dynamic R141 observation read U16 value 16, count 1, width 5 at +8 with the same parent-record token. This is consistent with BSID value 16; the output file was removed without payload read/hash, so no emitted bytes were independently checked.

The +8 field is not DialNorm. The +32 DialNorm value, its producer, any measurement/DRC join, output metadata, playback, and full end-to-end remain NOT_CAPTURED/UNBOUND/NOT_RUN/NOT_VERIFIED. This crosswalk does not supply a DialNorm implementation value.
