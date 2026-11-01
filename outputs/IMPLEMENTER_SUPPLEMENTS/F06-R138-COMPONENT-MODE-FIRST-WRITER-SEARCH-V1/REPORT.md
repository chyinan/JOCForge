# R138 component mode first-writer search

## Result

A hash-bound Ghidra direct-operand scan covered one Atmos image. It found 243 explicit writes at displacement `+0x54`. The selected component callback/open path has four reads and zero writes at that offset. None of the scanned writes is pointer-joined to the component instance passed to the callback; the R138 XML-to-member transfer remains `UNBOUND`.

One matching-offset parser writer was rejected: its caller passes a stack-local parser record, not the callback component instance. Equal displacement does not establish owner identity. The scan covers explicit direct memory operands only; it does not exclude aliasing, block-copy writes, or writers in other images.

## Evidence lineage

Incoming package `native-f06-r138-component-mode-direct-write-xref-unbound-addendum` manifest SHA-256: `44cb132c8623693e89cb612c59d42479cf38708a7a81f0f438b231c29ab905df`. Its checker passed in normal and optimized Python modes. The target image SHA-256 is `3d66bcec36031fd48e6565d15f05fea656642377ca4f8c98cdce1cce8b7e95d2`.

Parent Wash packages:

- `F06-R138-JOB-TO-MODE-FIELD-BOUNDARY-V1`, manifest `ba95fbedd155591bddf10eb11a7b21f3dc3c5dab414b52627c3a1e5c8dcbf74c`.
- `F06-R138-CONTEXT-MODE-OWNER-CONDITION-BOUNDARY-V1`, manifest `54ac629045a8d5b348f43f993894bbd9f0a707e6f165ff91d8e946ea41505810`.

The source package disclosed two out-of-scope vtable-related probes despite the task constraint. The package excludes those probes from its closure claim and reports their hashes. No project modification, DEE run, U16 capture, media read, or key/license read occurred.

## Remaining edge

Bind the actual component instance passed to the callback to its owner, then follow the first store to that same instance's `+0x54` back to the R138 job decoder. Do not treat the 243 whole-image candidates as component writers.
