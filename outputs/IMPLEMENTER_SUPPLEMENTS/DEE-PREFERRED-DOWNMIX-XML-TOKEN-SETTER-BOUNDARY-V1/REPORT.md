# DEE preferred-downmix XML token and setter boundary

## Bound static behavior

The encode_to_atmos_ddp/downmix/preferred_downmix_mode property is registered as an XML string with four exact tokens: loro, ltrt, ltrt-pl2, and not_indicated. The typed parser maps them to 32-bit enum-like values 0, 1, 2, and 3; an unknown token takes the invalid-map-key path. The static default literal candidate is loro, but effective behavior for omitted/empty XML was not tested.

The parser passes its mapped value to the typed setter. In the inspected construction path, the setter's optional callback is initialized to null. The setter then calls a helper that skips the callback and returns the pre-existing MixModeValue at +0x50; that return value is stored back at +0x50. Thus the examined edge does not establish that the parsed ordinal becomes the stored value. The initializer of the pre-existing code remains UNBOUND.

## First reader and remaining gap

A typed getter reads the stored +0x50 enum-like value, and a formatter maps that value back to one of the four tokens. No selected stage-processing callsite joins the getter/formatter to the audio or BSI consumer. Runtime object generation and actual XML value were not captured; no DEE job or audio run occurred.

This is an actionable static setter boundary, not proof that the option is inert for every construction path, nor proof of the effective output mode. The upstream package used an imported Ghidra project identity; the installed DLL was not rehashed. Its normal and python -O package checkers passed. No raw listing, source code, XML sample, media, PCM, payload, key, or license is included.
