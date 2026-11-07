# Individual record accessor bounds boundary

The selected individual-record accessor loads the U pointer-table start, sign-extends the ordinal, indexes one QWORD, and returns the resulting record pointer. The 12-byte accessor body contains no bounds comparison or conditional guard. It does not read the record's UID-derived physical slot field at +0x4C and does not read K_count.

This establishes that the accessor itself does not validate ordinal range or slot<K_count. The upstream ordinal/record validity, pointer-table capacity, actual allocation extent, physical slot invariant, and N-to-K relation remain UNBOUND. No runtime out-of-bounds behavior is claimed; this is one bounded accessor, not a whole-image scan.
