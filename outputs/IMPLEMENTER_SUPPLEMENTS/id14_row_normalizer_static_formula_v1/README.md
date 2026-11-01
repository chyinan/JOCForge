# ID14 six-byte row normalizer static formula

Ghidra identifies the classifier input as a signed byte at row +2, not row +3. The normalizer reads U16 +4 on the ordinary-tag run-comparison path. Row +3 is not read by this loop. Type 1 emits a mapped tag in the high seven bits and a run counter in the low byte; type 2 emits 0x8000 plus the row index. Valid row counts are 1..128; count and classifier errors are separate.

Twenty-nine synthetic input-only vectors are bound to static-formula expectations and independently recomputed by the Wash checker. No vector was executed in DEE. Two runtime row10 observations have raw +3=0xBA and +4=0, normalized 0x800A, but did not capture +2; they are not used to infer the raw tag or its ADM/EMDF meaning. This correction supersedes the prior washed supplement that incorrectly called +3 the classifier input.
