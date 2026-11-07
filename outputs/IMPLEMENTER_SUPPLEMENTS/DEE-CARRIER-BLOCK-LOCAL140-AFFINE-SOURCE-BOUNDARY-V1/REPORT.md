# `local_140` affine input-source boundary

Wash independently verified the sealed Ghidra 12.2 static package, its payload inventory, and all three cited parent manifest/seal pairs. The supplement omits raw Ghidra export and query source.

In the single inspected function `FUN_18006da40`, `param_3` is typed only as `longlong*` and arrives in R8/RDI. Stack local `local_140` (`[RBP-0x48]`) is written at RVA `0x6dbde` from `param_3[1] - 0x7586e + *param_3`, then read at `0x6e626` and passed as argument 2 to `FUN_1800729a0` at `0x6e635`. Other arguments include a child-table entry `+0xdf0`, `param_4`, and `param_5`.

This binds the local affine expression and its call argument, not `param_3`'s semantic owner/generation. The related candidate-argument path and plan-B `local_68` read remain separate; no typed pointer/generation join is established to the `puVar16` arena owner or plan-B descriptor. Preserve both joins as `UNBOUND`; the nearest break is the opaque `param_3` function input. Runtime/capture remain `NOT_RUN` under the existing keyless `NO_GO`.
