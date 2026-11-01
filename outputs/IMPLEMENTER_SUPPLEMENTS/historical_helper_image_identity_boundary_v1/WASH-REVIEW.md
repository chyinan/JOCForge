# Historical helper image identity boundary

This Wash addendum confirms two separate facts without joining them by co-location: the current DEE host image matches the hash-bound R138 host image and existing Ghidra import identity; the current helper candidate matches its own Ghidra import identity. The host's static command path builds a helper command with an input argument and passes it to a timeout-bounded execution wrapper. Its path prefix is selected dynamically.

The R138 log and execution record do not contain the child process's resolved helper path or image SHA-256. Therefore the host-to-helper command construction is statically bound for the hash-identified host, but the actual child helper launched in historical R138 remains UNBOUND. The current helper candidate's presence and hash cannot establish historical runtime identity.

No new DEE job, binary copy, decompilation, or private-key access occurred. The minimum missing evidence is a same-run child-process launch or file-open receipt with resolved helper path and SHA-256 tied to the R138 host process. Keep the time helper behavior candidate-specific until that edge is captured.
