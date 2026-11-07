# ADM physical-row to typed U16 layout-map rule

## Static rule

After exact CHNA TrackUID-to-AXML graph resolution, the physical slot is the resolved one-based CHNA track index minus one. The opened-source K row table stores each six-byte row at that physical slot, not at append order or group ordinal. The row count is the individual object count plus the sum of all group entry counts.

Each K row contains a semantic U16 input ID at +0, a U8 row tag at +2, and a U16 object/group ordinal at +4. In ordinary composite copy mode, the row position is preserved into the B descriptor. The selected layout-map builder accepts counts 1..128 and produces 128 U16 entries.

~~~text
Object row (tag == 255): map[r] = 0x8000 | (r & 0xff)
Bed row: map[r] = ((translate(tag) & 0x7f) << 8) | (group_bucket & 0xff)
~~~

The first typed consumers use the high bit to distinguish object rows. For ordinary copy mode, an object row’s low byte identifies physical slot r; its per-row object ordinal is not used by this U16 writer. A bed row’s low byte is the group bucket, not physical identity. Bed group bucket starts at 1 and advances on group-ordinal changes among bed rows; object rows do not reset the prior bed-group state.

A later canonical target map is a separate domain: slots 0..9 are 0x0101..0x0A01, and object suffix slots use 0x8000 | (target_slot + 1). Do not treat that target map as a physical-source map.

## 36/38 input-only count vectors

Both 36 and 38 fit the static typed map gate of 1..128. The authorized 36-row input-only metadata projection has 10 bed entries plus 26 object entries, K row storage 6*36 = 216 bytes, and a fixed 256-byte U16 map. The 38-row comparison has 10 beds plus 28 objects, K row storage 228 bytes, and the same 256-byte map.

These are metadata-only input projections, not native K/B/D runtime captures. Ordinary-copy versus extended-synthesis mode was not captured. Extended mode synthesizes a canonical prefix and changes row positions, so the ordinary-copy physical-slot formula cannot be promoted to that branch.

## Limits

The package does not join the converter-owned C map to the opened-source K map by equal counts or offsets. It does not prove native acceptance of a selected 36-row run, selected composite mode, or final audio output. No PCM, EC3, DBMD payload, key, or license content was read.
