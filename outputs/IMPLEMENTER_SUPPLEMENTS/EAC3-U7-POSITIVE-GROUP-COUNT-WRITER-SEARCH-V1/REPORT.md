# E-AC-3 U7 positive group-count writer search

The new bounded scan covers N-table write forms omitted by the earlier direct
MOV census. It followed dynamic-index/P-code paths, vector stores, and typed
bulk-helper callers around the D/N/code-row state. No positive assignment to
`N = u16(state+0x141c8+2*j)` was found in these scoped paths.

One related function receives the correct state base but reads neighboring
`+0xd4` for a seven-bit budget calculation, not N at `+0xd8`. Other paths
write pointer rows or operate through an intermediate table; the observed
vector stores are float-row writes/register spills. The earlier reset routine
still only proves zero initialization, and its exact owner/generation join to
the frame writer remains open. Unnamed helpers outside the fixed path set
were not scanned.

The best current model candidate remains `N=(D+2)//3`, with `3*N-D` tail
values of 2. This remains inference (confidence 0.78), not a native positive
assignment; the arithmetic-only vector `D=4 -> N=2, tail=2` is included. Use
it as a model-owned candidate for local vectors without treating this search
as product conformance or a reason to block best-effort implementation.

Ghidra static only; DEE runtime/encode were not run. No media, PCM, payload,
or key was read. Wash includes no source or decompilation.
