# Ghidra synthetic-kernel emulation addendum

The incoming Wash package has manifest SHA-256
`f3b19c63792632306bdd161dea639df6c7e21e890ba209f0c0f15ff3caf518e9`.
Its normal and optimized checkers pass and verify the target DLL identity,
Ghidra scripts, synthetic outputs, and access boundaries.

Two isolated Ghidra `EmulatorHelper` calls confirm the input/output vectors
recorded in `INPUT-VECTORS.json`: `FUN_1800a4d70` produced the local
delta-plus-two and tail-fill vector; `FUN_18008aea0` emitted one complete
MSB-first word and retained its residual cursor/accumulator state. Each
function had zero direct CALL instructions in the inspected range and
returned at a synthetic breakpoint. The input and output bytes are synthetic
scalar/kernel data, not PCM or a codec frame.

Use this as independent local-kernel confirmation alongside the existing
static delta-writer and fixed-width writer contracts. It can support focused
model vectors, but it does not establish function membership in the selected
JOC frame path, caller field order, complete AU validity, or product audio.
No FFmpeg/Audition parse or audio decode result is implied by this package.

The Wash supplement intentionally omits the Ghidra harness and inspection
source, the target binary, raw decompilation, media, PCM, encoded output, and
private key or license material.
