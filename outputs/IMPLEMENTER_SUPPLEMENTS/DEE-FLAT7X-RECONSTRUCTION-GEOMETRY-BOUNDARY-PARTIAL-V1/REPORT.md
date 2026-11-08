# flat7x reconstruction geometry boundary

## Scope

This supplement records the hash-bound target profile shape and a bounded Ghidra string-label query. It does not bind the profile selector to a native geometry consumer, coefficient matrix, or audio/BSI sink.

## Contract shape

The approved target contract records flat7x/profile index 1, seven non-LFE carriers ordered L/R/C/Ls/Rs/Lrs/Rrs, 15 coded objects, 15 reconstruction rows, and complexity index 16. The existing 5.1 profile is index 3/five_x_phase with five carriers, also 15 objects and complexity 16. Shared object and complexity counts do not identify the reconstruction geometry or imply that the profiles can share a matrix.

The seven carrier positions are contract positions, not verified native array indices. No carrier-to-row/object mapping is bound here.

## Native boundary

Ghidra 12.2 read-only/no-analysis exact defined-data queries for the string flat7x found zero matches in the two inspected images. The query did not inspect function bodies or branches and does not exclude numeric selectors or other representations. No typed link connects index 1 to a native profile object, seven-column geometry, coefficient consumer, matrix writer, or BSI consumer.

## Handoff

IMPLEMENTATION_READY=NO for the native reconstruction geometry. Treat profile shape as the approved target contract only. Native selector/type/stride, carrier-to-row/object mapping, and first matrix/BSI consumer remain UNBOUND; runtime remains NOT_RUN. No official MKA, EC3, mode21 payload, PCM, or DEE runtime was accessed.
