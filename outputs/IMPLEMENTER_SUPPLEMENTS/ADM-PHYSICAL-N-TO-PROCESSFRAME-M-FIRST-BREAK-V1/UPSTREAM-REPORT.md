# Physical input N to ProcessFrame M first-break

**Recipient:** Wash only  
**Result:** `ADM_N_METADATA_BOUND; N_TO_COMPOSITEMASTER_FORMAT_U16_UNBOUND; U16_TO_PLUGIN_PACKET_BOUND; PACKET_TO_CORE_M_UNBOUND; FIRST_OBSERVED_36_38_DIAGNOSTIC_SPLIT_AT_RIFF_PCM24_PARSE`

## Reused hash-bound inputs

The representative shape vectors use the existing seven-file metadata receipt `authorized_adm_admission_evidence` (manifest SHA-256 `6a2ccb7cc514736fd16b1acc3e8f6f2243a98cb429443bdbfd90410519d398d9`). Its auditor reads container metadata while skipping PCM data; the receipt records zero PCM-data bytes read and full input hashes before and after the audit.

- `First Love.wav`: WAVE channels, CHNA tracks, and DBMD tracks are all 36; input SHA-256 `db3ccb02260882bbd830d22c0ab207e6b8a6916743e87c939e2caa8fb4be4335`.
- `人是猫 - 张卡斯, 洛天依.wav`: those three metadata counts are all 38; input SHA-256 `7826dcbef37ef69642a6164c97a37bcfb3f917cea74aa0f0a30d76d0a825f1d1`.

These are physical-input N values from metadata receipts. They are not asserted to populate any downstream channel-count field.

## Nearest typed Atmos channel-count path

The sealed Ghidra 12.2 Atmos receipts bind `FUN_1801852E0` to a U16 `Atmos Channel Count` at `CompositeMaster.format+0x0A`. The constructor rejects zero and uses this U16 to size per-channel state. `FUN_180186870` copies that U16 to plugin packet `+0x08`; packet `+0x10` is a separate per-frame sample count. The packet process accepts 1,536 samples or zero/flush at `FUN_180179AB0`.

The nearest owner path is `FUN_1801852E0` → `FUN_1801828C0` → provider thunk `FUN_1804CBBC0` at `0x180183028`. The thunk dispatches the provider's virtual `+0xA0` slot and returns a 24-byte record; the first qword becomes the owner passed to `FUN_180338D30`, which yields the format descriptor. The concrete provider target and returned owner are not joined to the selected ADM reader/profile or to physical N. No typed store from N to `format+0x0A` was established. This is the first unresolved N-to-channel-count edge.

If the format U16 were 36 or 38, both values would pass the only checked constructor condition (`count != 0`) and would be copied into packet `+0x08`. This is a conditional code-path fact; the N-to-U16 identity remains `UNBOUND`.

## Core ProcessFrame M remains a separate domain

The reused core receipt defines M as the `U64` pointer-entry count in the core ProcessFrame view at `+0x20`; the pointer list is at `+0x08`. It is copied into a local descriptor before the existing configured capacity check. The host-frame receipt leaves the external caller and any reader/packet-to-core-frame transfer `UNBOUND`. The post-selection receipt also finds no static StageRunner caller to the core `Comp` process vtable. No typed path transfers the Atmos packet's `+0x08` channel U16 into core M, and no N-to-M formula is established.

## Where 36 and 38 first diverge in existing evidence

The hash-bound seven-input diagnostic reports six 36-channel files exiting at `RIFF_PCM24_PARSE`, while the sole 38-channel comparison reaches three local meter/frontend hops. For the representative pair above, `First Love.wav` has category `RIFF_PCM24_PARSE`; `人是猫 - 张卡斯, 洛天依.wav` has `PASS_LOCAL_MODEL_3_HOPS`, with no callback commit, no source-flags packet, and no encode. This is the first observed diagnostic-stage split. The exact native parser predicate responsible is `UNBOUND`; these statuses do not show 36/38 reaching different ProcessFrame M values.

The selected Atmos U16 nonzero check does not distinguish 36 from 38. The core capacity gate is a later M-domain check from the reused M-to-writer receipt; without N-to-M lineage, it cannot be named as the 36/38 split point.

## Scope

This package only combines existing hash-bound metadata and Ghidra 12.2 receipts; no new Ghidra database query or DEE run was needed. No original media payload, PCM, EC-3 payload, key, or license material was read. Vectors are input-only metadata/status shapes and conditional static facts; M and actual writer outcomes remain `UNBOUND`/`NOT_RUN`.
