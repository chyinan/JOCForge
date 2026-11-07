# Loudness metering-mode static caller boundary

## Disposition

Reference-only Wash evidence. It binds a DLL-local token map and a caller path from one processing object to the candidate string-to-code consumer. The XML schema StringType owner is not joined to that processing object, so no selected job value or metering effect is established.

## Static token map and consumer

The inspected local map contains `1770-4 → raw U32 0` and `1770-3 → raw U32 1`. Candidate consumer `FUN_1801535b0` reads a string-like value through getter slot `+0x30` at `param_1+0x210`, looks up the map, and writes a 32-bit candidate at `param_2+0x90`. The output signedness and semantic meaning are unbound.

## Processing caller edge

At `FUN_18014f1f0` callsite `0x18014f23e`, the observed branch `[RCX+0xf34]==0` calls `FUN_18014aba0`. `RCX` was copied to `RBX` and not rewritten before the call, so the current `this` pointer flows into that helper. `FUN_18014aba0` then calls `FUN_1801535b0` at `0x18014afb0` with its `param_1`. This binds a processing-object-to-consumer-receiver path only.

The XML schema StringType member at `FUN_180136220` `param_1+0x1f0` is not owner/generation joined to the processing `this` or getter receiver `param_1+0x210`. Selected provider generation, runtime token binding, metering/audio effect, and other metering modes remain `UNBOUND` or `NOT_RUN`.

The attached vectors contain only the two input tokens; static U32 candidates are separated from runtime outputs. No algorithm or parser behavior should be inferred from this package.
