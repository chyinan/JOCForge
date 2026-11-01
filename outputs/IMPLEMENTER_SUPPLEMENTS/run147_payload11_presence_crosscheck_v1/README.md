# Run147 payload 11 presence cross-check

The source Wash package and every payload hash were independently rechecked; its normal and optimized-mode checkers pass. The hash-identified Run147 reference output was independently rehashed to the same value in the source package. OpenJOC 0.20.1 parsed AU0 and reported payload ID 11 with group_id=0.

The cursor followed the documented configuration field order and found group_id_present=1 at bit offset 2 relative to the configuration prefix start. The absolute bit offset from AU0 is NOT_CAPTURED. The value matches the Run145 Wash observation; Run145 media was not reopened.

Both runs' actual loaded processing-module identities remain UNBOUND to the separately hash-identified static Ghidra target. The value match is an observed cross-run consistency point, not a universal producer rule or codec algorithm change. No payload body, PCM, or private key was read or saved; no DEE job or playback was run.
