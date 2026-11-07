# ADM physical-row to typed U16 layout-map rule — refreshed source package

## Static rule

After exact CHNA TrackUID-to-AXML graph resolution, physical slot is the resolved one-based CHNA track_index minus one. The opened-source K row table stores each six-byte row at that slot, not at append order or group ordinal. Count is individual object records plus the sum of group entry counts.

Each row carries a semantic U16 input ID at +0, U8 tag at +2 (255 marks object), and U16 object/group ordinal at +4. In ordinary composite-copy mode, the six-byte B row remains at the same slot r. The selected U16 layout-map builder accepts counts 1..128 and writes one U16 per row into a 128-entry / 256-byte map:

~~~text
object row: map[r] = 0x8000 | (r & 0xff)
bed row:    map[r] = ((translate(tag) & 0x7f) << 8) | (group_bucket & 0xff)
~~~

For bed rows, the group bucket starts at 1 and advances when the encountered bed group ordinal changes; object rows do not reset it. The first consumers interpret the high bit as object presence. In ordinary copy mode an object low byte identifies physical slot r; the bed low byte identifies the group bucket. A later canonical target map is a different identity domain: target prefix entries 0x0101..0x0A01 and object suffix 0x8000|(target_slot+1).

## 36/38 input-only projections

Both N=36 and N=38 pass the static 1..128 gate. For the authorized metadata-only N=36 projection there are 10 bed entries and 26 object entries; K table storage is 6*N=216 bytes. For N=38, the corresponding counts are 10 and 28, with 228 bytes. The U16 map allocation remains 256 bytes.

The 36/38 projections use exact UID-resolved input metadata, but are not native K/B/D captures. The ordinary-copy branch is a conditional model projection; extended synthesis changes row positions and its selected runtime branch is not captured. The literal symbol OAMDI was not present in the inspected code; this package names only the observed typed U16 layout map.

## Limits

The refreshed research package corrects a script/check payload and its manifest hash. Wash rehashed all 12 listed artifacts and seven Ghidra/input receipts. Three of ten older parent-manifest references were not located under the scanned research-audit roots, so they are not used to strengthen this claim. No DEE run, PCM/EC3/DBMD payload, key, or license content was read. Native values and selected mode remain NOT_CAPTURED.
