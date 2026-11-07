# Line DRC row-helper param_2 first call

Reference-only Wash supplement. It confirms forwarding of a selected-row pointer, not a DRC value. `IMPLEMENTATION_READY=NO`.

The sealed caller receipt binds `FUN_180182850.param_2` to the selected row pointer `param_4*0x20 + *param_3`. In the helper's bounded entry prefix, R8 and R9 are saved/reset while RDX is unchanged before the first row-related call at `0x180182882` to `FUN_1801431f0`. The selected-row pointer is therefore forwarded as that callee's second argument; the helper prefix reads/copies no row field first.

No typed owner/generation edge ties the selected row pointer to snapshot `+0xA0`, and the first callee body is out of scope. The snapshot pointer's presence at the upstream packer does not prove that the `+0xA0` value reaches this row path.