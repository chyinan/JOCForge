# Preferred-downmix encoder typed-value semantic vtable boundary

## Bound static target

The encoder option schema binds a `boost::program_options::typed_value<std::string>` created by `FUN_1400befa0`; its storage target pointer is at typed-value object `+0x10` and is registered as `DdpjocEncoderApp+0x360`. `FUN_140299aa0`/`FUN_1402992f0` retain the semantic pointer in option-description `+0x60`, which `FUN_14029cbf0` reads.

For the selected static vtable, slot `+0x28` resolves to `FUN_140296e20`. At the config merge call, it receives the semantic object, an intermediate option-record value slot, the parsed token range, and a mode byte. `FUN_140296e20` processes tokens into a temporary string vector and invokes typed-value slot `+0x48`, resolving to `FUN_1400bee30`, to build an intermediate parsed value.

## First break

`FUN_140296e20` does not read or write the typed-value storage pointer at object `+0x10`. Its value goes to an intermediate option record and config tree; no direct writer/notify edge to `this+0x360` is established. The runtime-selected option entry remains `NOT_CAPTURED`; argv-to-typed-storage remains `UNBOUND_STATIC`.

## Provenance

Source package `preferred_downmix_encoder_semantic_vtable_target_wash_v1` manifest SHA-256 `56b7585da3b2fe2cd5fc7336085a119e4950908180ce873ab1c2f876fb6165a5`; normal and `python -O` checker passed. Ghidra 12.2 static evidence; no DEE run, argv capture, media/PCM/payload/key/license access, or implementation inspection.
