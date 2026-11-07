# Preferred-downmix parsed config-tree writer boundary

## Bound static write path

The app calls `FUN_1402a3350(&local_220, this+0x18, 0)`. The receiver is the application config tree at `DdpjocEncoderApp+0x18`, distinct from Boost string storage at `this+0x360`. For a parsed option, `FUN_1402a3350` calls semantic vtable slot `+0x28` with the option-record value slot and parsed value range; the first observed write builds an intermediate value at `RBX+0x40`. The semantic handle is attached at `RBX+0x50`, and the record is inserted into an intermediate option tree.

## First break

The selected semantic object is not statically identified as the `encoder` typed value, and the config-tree merge path carries no pointer to `this+0x360`. The first break is the semantic-object identity and subsequent writer/notify edge from the app config tree to Boost typed storage. Therefore argv-to-`this+0x360` remains `UNBOUND_STATIC`; actual child argv and runtime selection remain uncaptured.

No DEE execution, media/PCM/payload/key/license access, or implementation-source inspection occurred.

## Provenance

Source package `preferred_downmix_encoder_config_tree_writer_boundary_wash_v1` manifest SHA-256 `eab045aee24d204d1a7b576aff79ec3c42d90d6db3af7bf5b244e702d5f983b4`; normal and `python -O` checker passed.
