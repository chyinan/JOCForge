# DEE DialNorm frame-record to BSI-U5 boundary

The selected static frame-preparation branch receives a per-frame record pointer at `frame_set + frame_index*0x38 + 0xd0`. Its body saves that pointer but shows no later read; the caller's subsequent stuffer call receives context-owned pointers, not the frame-record pointer. The first inspected five-bit writer on that route writes `convexpstr`; that field must not be relabeled DialNorm U5.

An alternate frame-preparation branch reads a context word at `+0x19140` into float-helper inputs, but the frame-record words examined come from separate context offsets `+0x19128` and `+0x19144`. This trace does not join the `+0x19140` value to core PCM coefficients or to a final BSI U5 writer. The active runtime branch was not observed.

The DialNorm-to-final-BSI-U5 join remains open. This package provides no evidence that DialNorm caused the measured 4-second PCM residual, or that it is PCM-invariant. It is a boundary finding only; keep audio matching focused on the validated window-table candidate and same-window trial.

Ghidra 12.2 static only. No DEE run, PCM/payload/key/license read, or output comparison occurred. No raw trace, source code, decompilation, or protected material is included.
