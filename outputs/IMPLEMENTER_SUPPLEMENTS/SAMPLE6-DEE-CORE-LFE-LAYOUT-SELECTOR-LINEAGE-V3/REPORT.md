# Sample 6 DEE core LFE row lineage — latest Wash supplement

Recipient: implementation evidence supplement  
Disposition: input metadata ordering and local row-copy order are supported; the selected-run row identity and LFE lane remain unbound.

## User-confirmed output identity

The user listened to the extracted DEE output and confirmed zero-based decoded output channel index 3 is the LFE/low-frequency track. Preserve `USER_CONFIRMED_LFE`; no further audition is needed. This does not identify which ADM row or DDP input row produced that output lane.

## Newly bound metadata and row-order facts

Wash independently verified the latest R4 package manifest, seal, SHA256SUMS, and all 19 payload sizes/hashes; its checker passed normally and under `python -O`.

The seven authorized ADM BWF metadata receipts cover six 36-channel inputs and one 38-channel input. Across 254 CHNA records, `track_index_1based - 1` matches the physical interleaved BWF slot with zero mismatches. This is a metadata-level slot rule; it does not show which physical row the selected native reader consumes.

The converter-local CHNA identity association and the DDP-local `FUN_18018d640` copy order are separately static-bound. The DDP copy keeps `input_row_table[i]` at internal row `i`; the identified first-copy callsite is RVA `0x18b29c`. Neither fact creates a typed same-run bridge from a BWF physical slot to a DDP input ordinal.

## Remaining audio-path join

The selected-run `reader row -> DDP input ordinal -> core descriptor/lane` identity and owner/generation join remain `UNBOUND`. The smallest existing-run observation is the DDP first-row-copy callsite at RVA `0x18b29c` together with the corresponding core `FUN_140350d00@0x140350f8b -> FUN_140317600` row/descriptor owner in the same run. The current package reports no new DEE run; numeric comparison remains `NOT_RUN`.

The matched core selector and layout remain unbound. Conditional layout tables are not a matched-sample result. This package does not identify a LFE DSP mismatch or authorize an audio algorithm change. The proposed input-only sine discriminator remains unexecuted with no expected output asserted.

## Upstream seal

- Package: `DEE-DDP-CORE-LFE-NUMERIC-EDGE-WASH-V1` (latest bridge revision)
- MANIFEST SHA-256: `86e655128b711faa43db8f2bd5bdceca57a14ab5ad86ef4fa992f12bad935106`
- package_seal.json SHA-256: `e66339e607ed6d21bec228fb8963adbef726a672ae2dc3884170611eb6613310`
- SHA256SUMS SHA-256: `2c369043c60c0d43f45d3ee5572e62d8751dcbc17cbf11ec8691620d44c0d0bd`
- All 19 payloads independently matched; normal and optimized checkers passed.
- Supersedes implementation supplement V2 manifest `a7b7bcb7f410c7875f5a3057ed6fd4e29a1e0f770855aaddc22a9a2419f8440a` / seal `9840159b81b777a7e530f3332e716eaeb3326dab8cf5e7e490f9a8718251312c`.
