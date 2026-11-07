# Line DRC snapshot+0xA0 row-helper boundary

Reference-only Wash supplement; it does not establish DRC effect or authorize an audio change. `IMPLEMENTATION_READY=NO`.

In the selected static branch `snapshot+0x9C == 0`, `FUN_1801764e0` calls `FUN_180182850(local_88, param_4*0x20 + *param_3, &local_170, &local_e8)`. The arguments are an output aggregate, a selected row pointer, and two local vector addresses. Neither the snapshot `+0xA0` U32 nor its address is an explicit argument. No typed owner/alias join connects the selected row pointer to `snapshot+0xA0`.

The first break is this call boundary. The row helper body, nested converter, later row consumers, codec/BSI input, runtime branch, and audio effect remain unbound or not run. The prior static `snapshot` pointer flow to the packer is not evidence that the `+0xA0` value reaches this helper.