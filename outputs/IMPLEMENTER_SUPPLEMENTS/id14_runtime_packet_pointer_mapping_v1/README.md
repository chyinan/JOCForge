# ID14 runtime packet correction and pending U6 input

This addendum corrects the previous outer/inner packet interpretation and records one pointer/scalar-only WinDbg run on the authorized Human Is Cat 1024 kbps profile.

The outer FUN_180179AB0 param3 is a process descriptor: slot 2 is the 1536-frame size. Slot 5 points to the inner metadata packet, passed through FUN_1801788a0 as FUN_180178DC0 param3. Only that inner packet has a U16 record count at +0x0a and row pointer at +0x10. The previous package's direct-param3 wording is superseded only for this pointer/count interpretation.

In the same thread, the run observed the inner packet count 38, selector enum 5, normal base 15, marker count 1, code13=16, code14=1, and count seed 15. The static ID14 writer path would encode count-1 as U6 value 14 if the instruction executes. The thread stopped before the writer instruction; no U6 output bits were written or read.

The task profile reports data_rate 1024, whose static map is enum 5. This is a same-job value cross-check, not a typed pointer identity from the host task object to the specific Priv instance. Packet row semantic names and a same-generation join from the frozen model EMDF list remain unbound. The Rust model's complexity value 16 is a separate input path and must not be equated with the native count seed 15.

The capture used output sink NUL; a 3704-byte task-local temporary file was removed without reading. No audio or payload bytes were read. Private license contents were not read.
