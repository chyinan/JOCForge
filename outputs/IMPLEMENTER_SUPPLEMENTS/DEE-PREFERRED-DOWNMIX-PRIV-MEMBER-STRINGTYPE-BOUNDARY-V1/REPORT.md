# Preferred-downmix Priv member StringType boundary

## Bound constructor and conversion path

The selected constructor chain passes the `Priv+0x20` subobject from `FUN_180135d40` into `FUN_180136220`. That initializer forms `param_1+0x1f0`, which is the same containing-object field at `Priv+0x210`, and passes it to `FUN_180138690`. The vptr writer assigns `dee::Types::StringType::vftable` to that member. The StringType vtable slot `+0x30` resolves to `FUN_180152490`, which converts/copies the member's text to an output string.

This closes the member type and vptr for this selected constructor chain. It does not connect this StringType to `preferred_downmix_mode` ordinals 0–3, the MixMode value wrapper, or an audio/BSI consumer.

## Boundary and provenance

The XML ordinal source is reused from its prior receipt. No DEE/XML runtime, media/PCM/payload/key/license access, or implementation inspection occurred. Source package `DEE-ATMOS-DDP-PREFERRED-DOWNMIX-PRIV-MEMBER-CONSTRUCTOR-BOUNDARY-WASH-V1` manifest SHA-256 `5a09cdaef80b89170cc39ad5f7a893651b23b94c78ec9f79926e9dbe57dfa366`; normal and `python -O` checks passed.
