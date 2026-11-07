# DI `+0x164` child-owner first break

Wash independently verified the one-caller Ghidra package and its parent hashes. Only sanitized facts are forwarded; raw trace, Ghidra scripts, and disassembly are omitted.

In `FUN_1801febe0`, `plVar8` selects a codec-parent child record through `param_1+0x10cc`; `lVar6=*plVar8`. The candidate reader `FUN_1801bcf40` receives `*(undefined8 *)(lVar6+0xdf0)` as `param_1`. This caller trace contains the pointer load but no assignment or constructor for `+0xdf0`; the loaded pointer target is opaque.

The nearest typed owner is the codec-parent child record `lVar6`. No typed pointer/owner/generation edge joins the opaque slot to Atmos engine subcontext `engine+0x1d0`. Bool-to-result causality stays `UNBOUND`. This is a one-caller boundary, not a global writer search or absence claim; runtime remains `NOT_RUN`.
