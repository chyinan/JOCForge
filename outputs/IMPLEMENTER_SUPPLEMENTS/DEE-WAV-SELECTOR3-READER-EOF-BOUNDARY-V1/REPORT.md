# WAV selector-3 reader boundary

## Static dispatch

The hash-bound Ghidra 12.2 package maps the suffix `wav` to registry selector 3. The lookup reads the callback from slot `param1+0x20` and calls it indirectly at RVA `0x4cc2c2`. The `MasterSourceImpl` reader path goes through `FUN_1805369a0 → FUN_1804cc430 → receiver vtable +0xd8`; that wrapper constructs an offset/count request and forwards it to storage.

## Exact open boundary

The package does not bind the runtime callback stored in the WAV registry slot to a concrete typed reader. The existing human_is_cat receipts describe two successful full-file runs but contain no registry slot pointer, callback address, or concrete storage receiver. Therefore the concrete `+0xd8` read implementation and its EOF comparison remain unbound. Whether the last requested sample `endSample-1` is read is not captured.

The parent package binds half-open range construction only; this addendum does not turn that into a final-sample inclusion rule. No DEE run, decode, PCM/payload/key/license read, or implementation contact occurred in this investigation.

