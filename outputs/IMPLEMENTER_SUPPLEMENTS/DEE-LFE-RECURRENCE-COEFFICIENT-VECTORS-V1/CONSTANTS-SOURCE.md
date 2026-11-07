# LFE recurrence coefficient vectors

This Wash-only input package contains only static data values read from the hash-bound `dee_ddp_encoder.exe` with Ghidra 12.2 PUBLIC. The exporter confirmed each requested range is in an initialized, non-executable memory block, then read only those ranges. No target instructions, decompilation, or executable bytes are included.

`INPUT-ONLY-VECTORS.json` stores each element's absolute address, little-endian 32-bit raw word, and decoded IEEE-754 binary32 value. Counts are: zero-branch table A 128, zero-branch table B 128, shared scalar vector 4, nonzero-branch table A 64, nonzero-branch table B 64.

The raw values are input-only data. This receipt does not bind the runtime selector or a sample's selected LFE branch and does not perform numeric parity or root-cause analysis. Those fields remain `UNBOUND`, `NOT_RUN`, or `NOT_ESTABLISHED` in the JSON and manifest.

No DEE execution, PCM or payload read, key/license access, or implementation-repository read occurred.
