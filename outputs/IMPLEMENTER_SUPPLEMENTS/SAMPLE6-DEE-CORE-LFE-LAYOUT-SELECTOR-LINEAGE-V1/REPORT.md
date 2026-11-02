# Human Cat core layout selector to LFE lane lineage

## Static producer path

The core input API receives six pointer/stride descriptors. A per-stream context supplies two U32 fields that the core copies into its layout selector and LFE-present state fields. The normal core branch normalizes these into the selector/count fields used by the input-lane mapper; the alternate core branch performs the same normalization separately. `FBW_count` is table-derived from the layout ID, and internal lanes `>= FBW_count` take the LFE main/companion path.

This binds the static selector producer and branch formula. The matched XML identifies `encode_to_atmos_ddp`, 1,024 kbps, and the input filename, but contains no sample-rate, channel-layout, or core layout-selector value. Existing CHNA evidence maps track index to the physical BWF slot, but does not observe which native core descriptor consumes that row. A separate DDP-local selector capture is not joined to CHNA or the core selector.

## Conditional mapping for descriptor slot 3

The table below is only a hypothetical mapping if ADM row 3 is first proven to be core descriptor slot 3. It does not identify the matched run's layout ID or prove that slot 3 is LFE. For layout IDs 4–7, slot 3 maps to an internal lane below `FBW_count`, while the LFE branch is at lane `FBW_count`.

Separately, the user has listened to the extracted DEE output and confirmed decoded output channel index 3 is the low-frequency/LFE track. This is a user-confirmed output identity, not a layout-order inference; it does not establish that ADM input row 3 fed a particular core descriptor or internal lane.

| Layout ID | FBW count | Descriptor row | Slot 3 internal lane |
|---:|---:|---|---:|
| 0 | 2 | `[0,2,5,-1,-1,-1]` | absent |
| 1 | 1 | `[1,5,-1,-1,-1,-1]` | absent |
| 2 | 2 | `[0,2,5,-1,-1,-1]` | absent |
| 3 | 3 | `[0,1,2,5,-1,-1]` | absent |
| 4 | 3 | `[0,2,3,5,-1,-1]` | 2 |
| 5 | 4 | `[0,1,2,3,5,-1]` | 3 |
| 6 | 4 | `[0,2,3,4,5,-1]` | 2 |
| 7 | 5 | `[0,1,2,3,4,5]` | 3 |

Matched core mode, outer selector/LFE-present values, layout ID, descriptor slot for ADM row 3, FBW count, internal LFE lane, and selected generation remain `UNBOUND`. No new DEE run or sample-byte read was made.
