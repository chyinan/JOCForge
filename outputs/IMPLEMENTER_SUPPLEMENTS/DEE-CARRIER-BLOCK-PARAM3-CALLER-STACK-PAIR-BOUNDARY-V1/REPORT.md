# `FUN_18006da40.param_3` caller stack-pair boundary

Wash independently verified the Ghidra 12.2 static package, all declared payload hashes, and the three parent package manifest/seal pairs. The forwarded supplement excludes the raw Ghidra export and query script.

The direct caller `FUN_18017bfc0` at callsite RVA `0x17c3c5` initializes adjacent `undefined8` stack locals `local_2c8` and `uStack_2c0` to zero, then passes `&local_2c8` as argument 3 to `FUN_18006da40`; the machine setup is `LEA R8,[RSP+0x30]`. The same address is also passed via `local_2a8` to `FUN_18006d550`, whose effects were not inspected.

This binds the caller-entry pointer to the stack pair, but not the pair's value when `FUN_18006da40` later reads `param_3`; the callee may write before that read. Therefore the `local_140` numeric value is `NOT_EVALUATED`. The `param_3` semantic owner, `puVar16` arena owner join, and Plan-B `local_68` descriptor join remain `UNBOUND`. The trace stops at the caller stack pair / opaque helper boundary; runtime remains `NOT_RUN`.
