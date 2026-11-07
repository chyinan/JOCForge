# Preferred-downmix config-tree first observer boundary

## Bound first observer

After parsed option records are merged into `DdpjocEncoderApp+0x18`, `FUN_1400b5310` calls `FUN_1400b46b0` with the config tree in `R8`. This observer performs required-option and conflicting-option checks. Its required-option helper looks up keys and reports missing entries; the conflict validator is a separate call.

The observer receives no `DdpjocEncoderApp+0x360` typed encoder storage pointer and does not pass a parsed encoder string to a typed setter/notify writer. This path validates config presence; it does not commit the encoder string to storage.

## Boundary

The config-tree-to-`this+0x360` writer edge remains `UNBOUND_STATIC`. Actual child argv and runtime branch remain `NOT_CAPTURED` / `NOT_RUN`. This is a bounded first-observer result, not a global absence claim.

## Provenance

Source package `preferred_downmix_encoder_config_tree_first_observer_wash_v1` manifest SHA-256 `1e4d12e16da249b43c3ca0acc2d0316d21ce85bc196ec06d1ef6d190171d6a71`; normal and `python -O` checker passed.
