# ADM physical channel count to ProcessFrame M boundary

## Hash-bound input metadata

The existing metadata-only receipt binds two authorized input shapes without reading PCM data:

- First Love.wav: WAVE, CHNA and DBMD each report 36 physical tracks.
- 人是猫 - 张卡斯, 洛天依.wav: WAVE, CHNA and DBMD each report 38 physical tracks.

These are physical metadata counts N. They are not assumed to populate any downstream count field.

## Nearest typed channel-count path

Ghidra 12.2 receipts bind FUN_1801852E0 to a U16 Atmos Channel Count at CompositeMaster.format+0x0A. The constructor rejects zero and uses this U16 to size per-channel state. FUN_180186870 copies it to the Atmos plugin packet +0x08; packet +0x10 separately carries frame sample count.

The nearest producer route reaches a dynamic CompositeMaster provider vtable slot +0xA0, which returns a 24-byte record. Its first qword becomes the owner passed into format construction. The concrete provider target and returned owner are not joined to the selected ADM reader/profile or physical N. The first unresolved edge is therefore N to format+0x0A, not a proven N-to-M conversion.

## Core ProcessFrame domain

The core ProcessFrame M is a separate U64 count of pointer entries at frame view +0x20. Its pointer list begins at +0x08. Existing receipts do not connect the Atmos packet's +0x08 U16 to core M, and no N-to-M formula is established.

The existing diagnostic receipts show six N=36 files stopping at RIFF_PCM24_PARSE, while the N=38 人是猫 comparison reaches three local meter/frontend hops. This is an observed local diagnostic split, not evidence of a native parser predicate or of distinct ProcessFrame M values.

Both 36 and 38 would pass the static Atmos count-nonzero check if those values reached format+0x0A. That source join remains unbound, so this conditional observation does not prove 36-track support or rejection.

