# speech_threshold first-break helper boundary

Reference-only Wash supplement. It records one helper argument/read boundary and does not establish an algorithmic read or audio effect. `IMPLEMENTATION_READY=NO`.

At `FUN_1801a8d00` callsite `0x1a8d7a`, `FUN_1801ad9d0` receives RCX=`*(engine+0x1f8)`, RDX=`*param_2` (the current record/output-carried value), and R8=`engine+0x228`. The helper's first data read is a 16-byte `MOVUPS` from `[RCX+0x100]` at RVA `0x1ad9f5`, then it copies first-argument data to the R8 output buffer.

This callsite passes neither the engine base nor engine `+0x48`. The first argument's owner/generation and any alias to engine `+0x48` remain unbound. The earlier U32 copy and 0 fallback rules do not prove a later engine read; threshold/audio effect remains `NOT_RUN`.