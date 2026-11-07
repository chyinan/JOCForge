# Line-mode DRC first row-converter boundary

## Bound selected branch

In the zero branch of the per-frame packer, FUN_1801764e0 passes a selected row pointer and local vector inputs to FUN_180182850. The snapshot+0xA0 U32 or its address is not an explicit argument, and the selected row pointer is not typed-joined to that scalar.

FUN_180182850 first initializes a generic output buffer, then calls a nested row-conversion child with param_2 and copies generic QWORD metadata to later output offsets. The child body, named output type, and the extra prototype formals whose sources do not appear in the caller's decompilation remain unbound. This is the next exact first-break.

## Limits

No codec/BSI input is reached in this slice. The other snapshot+0x9C branch, runtime branch selection, generation, clock, and downstream serialization remain unbound or uncaptured. The upstream package was hash-checked; Wash independently verified all seven payloads, SHA256SUMS, and its package seal. The upstream agent reported normal and python -O checks passing; no checker script is included. Raw trace excerpts are excluded from this supplement.
