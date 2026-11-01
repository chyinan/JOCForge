# ID14 raw-row member classification boundary

In one authorized, hash-bound run, a 38-row table with six-byte stride was copied into a process-state buffer, normalized into U16 values, and consumed by the marker test. For row index 10, the original and copied row both had byte +3 = 0xBA and U16 +4 = 0; the normalized consumer word was 0x800A and bit15 was taken.

Ghidra distinguishes the member reads: byte +3 is routed through a type-classification helper; U16 +4 is read only in a run-comparison branch. This does not establish that +4 alone caused 0x800A. The row semantic name and typed link from the selected ADM/EMDF field to this native row remain UNBOUND.

The marker count was one, producing code13=16/code14=1 in the count path. The U6 writer instruction did not execute; no bitstream was emitted or parsed. No payload contents were read.
