# Wash review: PCM24 reader to first F32 consumer

The bounded Ghidra chain from the current hash-bound converter reader reaches a conditional three-byte source-sample branch. That branch assembles high bits and stores through a `short*` using a carry predicate; it is a conditional 24-to-16-bit narrowing fact, not a proven F32 normalization rule. The separate routine that returns `float` belongs to a four-byte input path and has no call edge from the three-byte branch.

No typed transfer connects this short intermediate to the DDP F32 row owner or first numeric DSP consumer. The sample-6 run did not establish that this branch was selected. Scale, precision and rounding remain unbound. Do not use the short narrowing expression as the PCM25’F32 algorithm and do not import an external encoder's scale.

The approved JOCForge input spec separately defines U24LE signed PCM24 reading and byte layout. For audio equivalence, the remaining minimum DEE observation is one signed nonzero input sample and its same-generation first-DSP F32 bit pattern with a typed row/owner join; do not persist samples.
