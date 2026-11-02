# Sample 6 DEE core LFE selector lineage — superseding Wash supplement

Recipient: implementation evidence supplement  
Disposition: retain the confirmed output identity; do not infer the input-row or internal-lane mapping from it.

## User-confirmed output identity

The user listened to the extracted DEE output and confirmed zero-based decoded output channel index 3 is the LFE/low-frequency track. Record this as `USER_CONFIRMED_LFE`. No further listening confirmation is needed. This identifies the decoded output channel; the mapping from ADM physical row 3 to a DEE core descriptor and internal lane is a separate, still-unbound fact.

## Verified static path

Wash independently verified the final upstream R4 package against its manifest, seal, `SHA256SUMS`, and all 18 payload hashes and sizes. R4 reports its own checker passed in normal Python and with `python -O`.

For `dee_ddp_encoder.exe` (SHA-256 `0cf23520799f52323b0994e5fcc16924da7b027a5b83a0cff3b4f06792c18532`), Ghidra 12.2 static evidence identifies six input descriptors, each 16 bytes, with an input pointer at offset 0 and a U16 sample stride at offset `+0x0c`. At `FUN_140350d00@0x140350f8b -> FUN_140317600`, the processing path receives the descriptors and copies outer context fields `lVar9+0x40/+0x44` to core state `+0x2c/+0x30`. Both core branches normalize them into layout/LFE-present fields at `+0x1912c/+0x1912e`. The layout table selects input slots; lanes at or above `FBW_count` use the LFE main/companion branch.

The mapping for descriptor slot 3 is conditional. Layout IDs 4–7 map that slot to internal lane 2 or 3, below `FBW_count`; that table does not show that ADM row 3 supplied descriptor slot 3, which layout the matched run used, or that the matched run selected the LFE branch.

The static core algorithm path is identified but is not a matched-run comparison: `FUN_14036e350` reads at most input indices 0–255 and performs 64 outer symmetric-pair iterations. The main and companion LFE stages each produce seven F32 values per block. This traversal shape alone does not establish semantic parity or a PCM root cause. The matched descriptor, layout, stride/history slot, coefficients, and runtime values remain unbound. No algorithm correction follows from this package.

## Exact remaining join

The smallest existing-run observation point is the already identified call `FUN_140350d00@0x140350f8b -> FUN_140317600`. A same-call metadata record needs the six descriptor owner/source-slot/stride values, `lVar9+0x40/+0x44`, and normalized selector/count fields. Existing receipts lack that record. Therefore ADM physical row 3 → core descriptor slot → internal LFE lane remains `UNBOUND`; the user's confirmed output-channel identity remains `USER_CONFIRMED_LFE`.

The matched XML binds `encode_to_atmos_ddp`, 1024 kbps, and the input filename. It contains no core mode, layout selector, sample rate, or channel-layout fields. The upstream package did not locally rehash the ADM or DEE output; it did locally rehash the matched XML. Its proposed synthetic vector is not executed and has no asserted expected output.

No new DEE run was performed. No raw PCM, audio payload, or private key/license material was read or included. No Ghidra source/decompilation artifacts are included in this supplement.

## Upstream package seal

- Package: `DEE-DDP-CORE-LFE-NUMERIC-EDGE-WASH-V1`
- MANIFEST SHA-256: `883dfbf0bee6e0bcc52fa232109b3723d83a841b97551b48abacd20b335bdc2e`
- package_seal.json SHA-256: `5b54d7a1c43aa8c022415141fe0064f9f7c2fb970c84f70e2401b340cdb310c8`
- SHA256SUMS SHA-256: `2adaf949d6bb0341a0c21de57e6ae345642431a05358e63d43b8be4c3480fbc2`
- Payloads: 18; Wash independently rechecked all member lengths and hashes.
- This source supersedes manifest `d4043721c4e4a00ad3013259ba23430f751f0245924f437333578c4c2ee718eb` / seal `44e2e7f7242025c8c022650f8e17936e74263286822dc3ff2b9b5669388c7973` and implementation supplement V1 manifest `b4af51070e7f52242e04535c4abefc95d0bfc31086e264692af7645c600897f2` / seal `1e62aeaa0dc9eef1ef47679557b8c8c5d0acf37711233702a6c01016f3abf4f3`.
