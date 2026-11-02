# E-AC-3 U7 group-count reset and model candidate

Ghidra binds a six-by-five zero-initialization candidate for signed D and
unsigned N fields in the `state + 0x140f0` subobject. The candidate's call
passes `RBX + 0x140f0`; its stores zero N at `+0x141c8 + 2*j` and D at
`+0x1418c + 10*block + 2*lane`. The same field namespace is read by the U7
producer/writer contracts. A direct same-object/generation edge to the frame
writer has not been observed, so keep that reset ownership marked UNBOUND.

A bounded direct-MOV scan found no positive N writer, but explicitly excludes
dynamic-index, vector, and bulk-copy stores. The best current model candidate
is `N = ceil(D/3)`, followed by `3*N-D` tail values of 2. Five arithmetic-only
vectors cover D=0,1,3,4,6. This relation is still inference-only (confidence
0.78), not a native assignment. It is usable as a model-owned candidate and
does not require waiting for the remaining store-search boundary to build
focused model vectors.

DEE runtime, encoding, media, PCM, payload, and key access were NOT_RUN or
excluded. This supplement contains no binary or decompilation and makes no
complete-frame or output-parse claim.
