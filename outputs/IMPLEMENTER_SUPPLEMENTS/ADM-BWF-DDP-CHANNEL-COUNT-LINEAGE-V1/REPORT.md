# ADM BWF reader count to DDP channel-count boundary

Recipient: implementation evidence supplement  
Disposition: local count paths are typed and dynamic; the cross-component count transfer is not established.

## Converter-side ADM BWF reader

Ghidra 12.2 static evidence binds the selected `atmos_mezz` reader's count getter to a U64 value at reader/storage object `+0x18`. The selected load path compares that value with the CHNA header's U16 declared track count. A separate expected-count field at converter owner `+0x6368` is caller-supplied and compared to the getter. A guarded branch rejects counts of 129 or more; no fixed 36 or 38 comparison was found in the checked reader functions.

## DDP-side configured count and descriptor

In the selected DDP Atmos path, a nested config U16 at `(*(qword *)(param3+0x28))+0xA` is zero-extended into processor `+0x20`. That count drives a `MultiChannelBuffer` row-pointer allocation (`count * 8` bytes) and float storage (`count * frame_capacity * 4` bytes). For nonzero-frame calls, the descriptor's U64 channel count at `+0x08` must equal processor `+0x20`; its `+0x00` is the source row-pointer table and `+0x10` is frame count. No explicit channel-count upper bound was found in the selected allocator/initializer functions.

## Critical non-join

The converter reader count, converter caller expected-count, DDP nested-config U16, and process descriptor count are distinct typed fields. The available evidence does not bind a host-side writer/argument transfer from the selected ADM reader into the DDP nested config or descriptor for the same owner/job generation. A prior single-run observation of 38 DDP rows did not hit the ADM reader breakpoints and cannot prove this transfer. The 36-channel ADM metadata check proves the file has consistent 36-channel/CHNA/AXML declarations, not that this DDP path accepts it end to end.

An adjacent core-image boundary found a 16-entry process-frame pointer staging array before a later count/capacity check. That is a separate boundary and has no typed join to the converter's dynamic DDP row allocator. Do not treat the dynamic DDP allocation as proof that a 36/38 process-frame count is safe in that other wrapper.

No DEE run or runtime capture occurred for this package. No PCM, encoded payload, key/license, source code, or decompilation text is included. The upstream package supplied no separate input-only numeric vector; this supplement adds none.

## Upstream seal

- Package: `adm_bwf_ddp_channel_count_lineage_wash_v1`
- evidence_manifest SHA-256: `851a03dca222f40c7b9eb102a33d1bad7d7b942151314941a84c396a0c140402`
- package_seal.json SHA-256: `4a13d78cfbe2ed3adb698ada8a97c0963afa1615afbf4b788af83d774dfdfc0b`
- Five payloads independently matched the seal's length/hash list; upstream normal and `python -O` checks passed.
