# DI measurement configuration field initializer

Ghidra 12.2 static evidence binds the meter configuration byte at +0x14 to an immediate value of 1 written before the setup Bool is read and written again as 1 afterward. The setup Bool at +0x94 is copied to the normalizer input at +0x04 and affects output configuration byte +0x00. It is not the producer of byte +0x14 in this inspected function.

A downstream constructor reads configuration +0x14 and copies its nonzero state into meter state. Runtime branch selection and any final measurement or audio effect remain unobserved. This package reuses sealed read-only traces; no new Ghidra run, DEE run, runtime capture, media/PCM/payload access, or implementation change occurred.
