# R6 F13/JOC count crosswalk

Recipient: implementation evidence supplement  
Disposition: the native coefficient-slot extent is mode-table-derived; no ADM/object-count source join is established.

## Input count domains

Existing authorized metadata receipts distinguish the 36-channel `潜在的なアイ.wav` input (36 CHNA rows/UIDs; AXML bed/object decomposition not recorded) from the 38-channel `人是猫` input (38 CHNA rows/UIDs; 10 DirectSpeakers bed rows and 28 object-associated tracks). These counts remain separate dimensions. The 38-input `10 + 28 = 38` equality is a metadata cardinality check, not proof that the native serializer consumes those rows as an object count.

## Native static count chain

Ghidra 12.2 evidence in `dee_audio_filter_ddp_atmos.dll` (SHA-256 `3d66bcec36031fd48e6565d15f05fea656642377ca4f8c98cdce1cce8b7e95d2`) binds a U32 dispatcher-record field at `record+0x34` copied to U32 state `+0x2c`. A lookup table at RVA `0x83fa60` derives a U16 coefficient-slot extent at `state+0x191c8`; the block path consumes it as a loop extent and passes it into `FUN_1800a3710`. The same block invokes `FUN_1800cb0d0`, which contains a candidate two-value, seven-bit writer path.

The `record+0x34` field is inferred to be a mode code from its table lookup and use; it is not named as an ADM track or object count. The transformed pointer table and candidate packer use different offsets, so pointer alias and the exact serialized F13/EMDF count remain `UNBOUND`.

## What this says about arbitrary track input

This static chain does not normalize ADM track counts to an object count and does not show that source row counts differ only at reader admission. Keep PCM channels, CHNA rows, bed rows, object-associated tracks, DDP frame channels, coefficient slots, and EMDF count separate. No track-count-to-mode/slot edge is bound, and no audio algorithm conclusion follows.

No DEE run, PCM/data-chunk or encoded-payload read, or key/license access occurred. The upstream input discriminator is symbolic metadata only; it has no native execution or expected output values.

## Upstream seal

- Package: `r6_f13_joc_count_crosswalk_wash_v1`
- MANIFEST.json SHA-256: `3135b65e1113b013f6bc4a364c9483bb9c7bde353bc819bb07bc1bbce6c79b43`
- Five payload hashes and sizes independently matched the manifest; normal and `python -O` upstream checker passes were reproduced.
