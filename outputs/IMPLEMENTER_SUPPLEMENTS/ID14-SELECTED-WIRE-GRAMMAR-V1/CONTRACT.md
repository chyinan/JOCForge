# Selected ID14 descriptor and coefficient bit grammar

Status: approved clean-model grammar with a synthetic table-bound exercise; this is not a recovered native reader ABI. Native descriptor kind/mode semantic mapping in the older header-only native evidence remains STATIC_ONLY; the selected controlled native coefficient-writer slice separately exercised 15 synthetic cases. Do not infer the native descriptor layout or a general native enum from the local model.

## Scope and sources

The directly applicable approved rules are `IMPLEMENTER_PACKAGE/specs/05-joc-quantization-and-wire.md` sections F05Q.1 and F05Q.3–.6; `specs/05-joc-huffman-codebooks.md` F05QH.2–.4; `specs/05-live-joc-coordinator.md` F05JL.1, .5 and .6; and `parameters/joc-huffman-v1.json` (raw SHA-256 `515d635adc6099493dfd27f787bdf2b9f19a4b4bb1180860b0e820a23fb5d5ae`). F05Q is a complete writer wire definition for the selected local model; the algorithm below is its bounded inverse. F05Q-ID14-HEADER-FRAMING alone observes wire fields but explicitly leaves native kind/mode meanings STATIC_ONLY. F13C-ID14-COEFFICIENT-PAYLOAD-WRITER-V1 records controlled native writer vectors, not native semantic reader or real audio evidence.

## Header and descriptors

All fields and Huffman codewords are read MSB-first from a bounded bitstream. Read header U3 downmix, U6 object_count_minus_one, U3 extension_config, U8 clip_gain_index and U10 sequence. For selected F05JL require downmix3→five non-LFE carriers, object_count=15 (encoded14), extension0, clip index 128..255 and a sequence matching the popped producer receipt. This exact header is 30 bits; no byte alignment follows it.

Read all 15 descriptors in ascending object order before reading any coefficient data. Each begins present U1. For generic F05Q present=0 has no descriptor tail or coefficient data; selected F05JL requires all 15 present, including inactive spatial slots. For present=1 read in order:

1. `band_enum` U3. Map 0..7 to `[1,3,5,7,9,12,15,23]` bands. Selected begins at enum5 (12 bands) and F05JL retry can lower it to 0; selected results are 0..5. Enums6/7 belong to the generic leaf but are not reachable in this selected route. Do not reject a permitted retry result merely because enum is not5.
2. `mode` U2: 0 full coarse; 1 full fine; 2 sparse coarse; 3 sparse fine. Equivalently the first bit is sparse and the second fine. The selected initial mode derives from its approved precision provider and may change with retry; do not assume every accepted writer is mode1.
3. `steep` U1 and `point_count_minus_one` U1. If steep=0 no offset fields are transmitted. If steep=1 read one U5 per point in point order; these are zero-based slots 0..23, not a channel index. Reject offset >=24. Generic F05Q leaf supports one or two points; the selected F05J live producer supports only one point and must reject point_count=2, rather than duplicate a point or silently skip it. Do not interpret offset as F12 carrier block.

## Coded data

After every descriptor, traverse objects ascending; skip absent objects in generic parsing. For each present object traverse its points ascending. Let channel_count=5 and bands from that object's enum. The table names and state reset are:

| mode | S | coefficient table | initial previous | traversal |
|---|---:|---|---:|---|
| 0 | 96 | full_coarse | 48 per point and channel | point→channel 0..4→band |
| 1 | 192 | full_fine | 96 per point and channel | point→channel 0..4→band |
| 2 | 96 | sparse_coarse | 50 per point | point→all channel positions→all band coefficients |
| 3 | 192 | sparse_fine | 100 per point | point→all channel positions→all band coefficients |

For full, reset previous at each point/channel start; for each band read one codeword from the specified coefficient table, obtaining symbol in 0..S-1, calculate `q=(previous+symbol) mod S` and set previous=q. For sparse, at each point read the first selected channel directly as U3 and require 0..4. For bands 1..B-1 read one `position_5` Huffman symbol and recover `channel[b]=(channel[b-1]+symbol) mod 5`. After **all** B channel positions for that point, reset coefficient previous to 50 or100 and read B coefficient symbols from the respective sparse table using the same modulo update. Associate each recovered coefficient with its recovered channel and band. Never interleave one position and one coefficient per band, and never carry previous to another point, channel, object or callback. `position_7` is supplied for the generic seven-channel leaf, but selected downmix3 uses `position_5`.

Use the exact shipped symbol/code/bit_length mappings, not canonical codes reconstructed from lengths. Validate all six fixed tables and symbol counts 96,192,96,192,5,7, each symbol unique and in its declared range; code lengths 1..24, values fit widths, codewords distinct/prefix-free and complete. Read one bit at a time until a leaf, returning the symbol; start the next symbol at root. A bitstream ending at an internal node, invalid table/branch or wrong table is a failure. `q` is a quantized code, not an inferred original F32 coefficient or audio value. Source provider value identities and exact per-table entries remain the supplied `joc-huffman-v1.json`.

After exactly all expected codes, require only 0..7 remaining pad bits and all zero; reject unexplained extra bytes, nonzero padding, truncated fields/offsets/codes, arithmetic overflow, selected object/count/downmix/extension mismatch, incorrect sequence or producer/body identity. Selected producer capacity <=372 bytes and coefficient scratch <=4344 bits are producer constraints. For a separately supplied parser input enforce the selected profile's declared body bound and bounded cursor, but do not interpret padding as coefficient bits. Transaction staging must leave queue and committed state unchanged if this validation fails.

## Evidence and limits

The frozen F05Q Huffman consumer model confirms all four mode/table and traversal rules. The previously washed controlled native writer slice reports zero-coefficient selected downmix3/15 objects/enum5/full-fine/one point with 150-bit header+descriptors and 900 coefficient bits, producing 1050 valid bits padded to132 bytes; it uses synthetic non-audio input and does not establish an entire product path.

An independently written research prototype generated and parsed four one-point, fifteen-object cases from explicit selected shapes and the shipped table entries; all were within372 bytes, recovered the generated quantized codes/channels, and rejected truncation, both with normal Python and `-O`. This is a same-table roundtrip, not independent golden/native parity, an exhaustive Huffman validation, real ADM, or the full EMDF parser. It uses constant coefficient codes around each mode's initial previous and varying sparse positions. The input-only recipes below require future tests for retry band values, absent objects at the generic leaf, steep offsets, invalid selected two-point and nonzero padding. Full produced current ID14/F13/F12 composition and real audio remain NOT_RUN.
