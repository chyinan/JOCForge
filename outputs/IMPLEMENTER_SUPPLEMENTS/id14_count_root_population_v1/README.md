# ID14 count-root population: scoped static rule

This package washes a hash-verified Ghidra 12.2 follow-up for the plugin query-buffer root and the normal-path code-13/code-14 values feeding its count constructor.

For the normal selected plugin path, the typed parser writes U32 code 13 to root +0xc170 (default 11) and U32 code 14 to root +0xc174 (default 0). The property-list builder supplies (base + high-bit-marked U16 metadata-record count, marked-record count). The count constructor subtracts code 14 from code 13, yielding base 11 for a zero adapter selector or base 15 for a nonzero selector; accepted count seeds are 1..15.

The package also records a static query-buffer clearing/capacity proof. It is not a runtime pointer capture. The special-mode branch is separate and is not generalized. This evidence does not map the selector to host task fields, identify the selected ADM/OAMD generation, or prove a dynamic same-run join. Native capture was NOT_RUN.

Implementation may use only the scoped normal-path formula when an existing approved model input already supplies the corresponding selector. Do not add a user-facing preferred/parameter13 prompt based on these property codes. The relation between the native count seed and JOC body object_count is not established here.

Only sanitized summaries and synthetic input-only vectors are included. No Ghidra decompilation, scripts, source binary, media, PCM, payload, bitstream, or private key/license contents are included.
