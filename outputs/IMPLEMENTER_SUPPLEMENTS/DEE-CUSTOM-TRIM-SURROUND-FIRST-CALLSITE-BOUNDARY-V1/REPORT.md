# Surround trim candidate: first callsite boundary

Reference-only Wash supplement; it does not establish the candidate as a selected trim consumer. `IMPLEMENTATION_READY=NO`.

The bounded Ghidra xref receipt identifies the untyped slot at `0x180843b00` as containing the address `0x1800b5c60`. The exact references-to query for the slot finds no code/data references; the function entry has zero code references and two data references (runtime metadata and the slot). Thus the available xref model supplies no indirect callsite or call arguments, and cannot establish that `RDI` owns the keyed `StringType` at `+0x968`.

The first break remains slot → indirect use/callsite → typed RDI owner. This is a bounded reference-model result, not a global absence claim. Runtime owner/generation, selected trim consumer, units, and effects remain unbound/not run.