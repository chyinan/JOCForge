# Native OAMDI key path and F06OF bridge boundary

## Bounded native path

Static analysis of the hash-identified DEE Atmos component found two separate key surfaces. A U8 candidate at byte 2 of each six-byte channel descriptor is collected into a native index list. A separate U16 key at the start of each 48-byte OAMDI event record is bounds-checked, used to select a native per-object owner, and carried into the queued event. The U16 key-to-native-owner lookup is established for this static path.

The descriptor U8 and event U16 have not been joined to each other. Neither has been typed as an ADM AudioObjectID, F06OF SelectedGlobalTargetId, TargetSlotIndex, control16 word, or callback generation. The table-B literal-row path is separate and remains unjoined to this OAMDI path.

## Approved model path

The approved model maps F03G typed target IDs into fixed TargetSlotIndex values, preserves per-slot history/producer lineage in F05H/F05R56, and maps slot s through the named F06OF adapter to control16 row 1+s. A hash-bound one-hot model vector distinguishes two IDs and the checker derives the expected model assignment/projection. This is model-only: the vector was not freshly replayed against native DEE.

## Open native lineage

No native capture or typed adapter joins the OAMDI U16 owner key, descriptor index, table-B property, or native object state to the approved model target ID/slot/control16 word/generation. These cross-domain joins remain `UNBOUND` or `NOT_CAPTURED`. Do not infer identity from equal integers, table row/stride, or cross-run hashes. Product policy is unchanged; this package establishes no native F06OF producer closure.
