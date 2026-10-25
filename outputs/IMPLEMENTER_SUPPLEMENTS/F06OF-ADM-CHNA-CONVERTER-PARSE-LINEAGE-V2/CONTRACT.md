# F06OF ADM/CHNA converter lineage with bed-value arithmetic

This V2 supplement supersedes the bed-field `PARTIAL` note in V1 using a hash-bound constructor-argument audit. It does not close the later converter-to-frame-descriptor connection.

## Input label and CHNA slots

The source remains ADM BWF. `atmos_mezz` is a DEE XML node label for this source path, not a different input type. F01E uses the `adm` node when present and otherwise falls back to `atmos_mezz`.

Each CHNA table entry has a 40-byte stride, a leading U16 track index, and a 12-byte AudioTrackUID at row offset 2. The converter validates UID uniqueness and track-index bounds/coverage, constructs a UID-to-track-index lookup, and uses `track_index - 1` as a storage-vector slot. This track slot is distinct from the decoded channel input value.

## Object and bed channel values

- For an ADM object `AudioObjectID` in the accepted `AO_%04x` form, the numeric suffix is decoded and the input ID is `suffix - 0x1001`. The descriptor keeps this channel input ID separate from `track_index - 1`.
- For each ADM bed channel, the converter decodes the bed `AudioObjectID`, maps its AudioTrackUID through CHNA, and supplies a zero-based bed-channel ordinal. The bed channel constructor receives a 32-bit third argument equal to the arithmetic sum `(int)decoded_channel_input_id + (int)zero_based_bed_channel_ordinal`; it receives a separate fourth argument `(int)track_index - 1`.
- The Bed::Channel record stores the third argument at its channel-input value field and the fourth argument at a separate track-slot field. The converter's U32 map write uses `track_index - 1` as the slot and the channel-record value field as the value. Thus the bed value is the stated arithmetic sum, not a tuple or packed bitfield.
- `AdmFileStorage` reads that channel-record value as its lookup key. The converter-local object/bed/CHNA-to-storage lookup is statically closed; the source says nothing about PCM consumption by the downstream measurement frontend.

Do not merge UID, CHNA track slot, decoded object channel input ID, bed ordinal, or their bed arithmetic result by numeric equality. In particular, bed source-map values include the ordinal addend, whereas object source-map values use the decoded ID directly.

## Remaining frame boundary

The static chain is closed through the converter-local storage lookup, but it is not joined to the selected measurement frontend's `AtmosProcessor` descriptor, physical frame-row order, PCM range/cursor, or runtime owner/generation. The earlier source-map strings and the separate spatial metadata rows are not substitutes. Keep those edges `PARTIAL/UNBOUND`; `IMPLEMENTATION_READY=NO`.

This is hash-bound static evidence only. No converter or native execution occurred; the original primary PCM remains `NOT_REHASHED`.
