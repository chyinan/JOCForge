# ProcessFrame selected profile index producer

## Key-1 override and static default

`FUN_14031FAE0` writes selected profile index as signed I32 into the builder block at `+0x3C` (the same root's `+0xC13C`). It scans custom `(I32 key, I32 value)` pairs; a key-1 record copies its value directly, without remapping. If key 1 is absent and the image-level flag `DAT_1405EDA90==1`, the static branch copies `DAT_1405EDA9C`; the hash-matched image values are flag 1 and default index 0. If neither branch writes, the output depends on the caller-supplied root-clear extent.

`FUN_1403210C0` treats that I32 as a direct profile-table index. The bounded supported list is `[0,1,3,4]`, with row counts `[13,11,10,5]`; index 2 and indexes 5..13 have no rows. The default path statically selects index 0 when taken, but the actual job's key-1 override and runtime profile are `NOT_CAPTURED`.

## Conditional interaction with key-2 capacity

From the separately sealed key-2 receipt, profile index 0 has a key-2 default C=11 with allowed custom range 1..11. Thus, **if** key 1 is absent, the static default branch is selected, and the same root initialization supplies the expected key-2 value, C would be 11. The ProcessFrame capacity gate rejects when `M>C`; however, actual profile, root extent, active C, and `ADM N→M` are not joined. This does not prove that 36/38 inputs are rejected or supported.

## Provenance

Upstream manifest SHA-256 `e50c225e4ef465271c93db438a264880714e8f52603d08de42cd471ae3589458`; upstream package-seal SHA-256 `8e5c4dcf212fee5406a11d7d34eb2f031fe6fe92ef54e967d69be75f14bfc679`. The source verifier passed normally and with `python -O`; Wash rechecked the manifest/seal hashes. No DEE run or media/PCM/payload/key/license access occurred.
