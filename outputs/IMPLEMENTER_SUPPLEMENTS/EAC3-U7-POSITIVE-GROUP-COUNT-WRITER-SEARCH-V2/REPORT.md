# E-AC-3 U7 group-count full-image inventory

The follow-up inventory searched the full loaded Ghidra image (1,276,863
instructions) for literal field displacement `0x141c8` and subobject base
`0x140f0`. It found zero literal field-displacement occurrences and 21 base
LEA references; all 21 and their 10 resolved argument-forwarding occurrences
are within the previously scanned code range. No outside-range callee,
unresolved indirect base call, forwarded `state+0xd8` pointer, or new writer
was surfaced.

This closes the full-image expression-based inventory, not all possible
indirect pointer provenance: a base loaded without either literal displacement
can evade this search. No positive native N assignment was found. The
best-supported model candidate remains `N=(D+2)//3`, with `3*N-D` tail
values of 2, confidence 0.78 and inference-only grade. The D=4 -> N=2,
tail=2 vector is arithmetic-only and not a native observation.

Use the candidate as a model-owned implementation rule for focused tests; do
not wait for an absolute absence proof, and do not claim native positive-N
writer behavior or same-generation ownership from this inventory.
