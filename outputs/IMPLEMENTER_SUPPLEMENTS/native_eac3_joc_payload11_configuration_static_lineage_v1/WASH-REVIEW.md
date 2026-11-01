# Native Type 11 EMDF configuration writer boundary

This Wash addendum seals the static Type 11 descriptor and conditional configuration-writer rules from the hash-identified DDP/Atmos module. It distinguishes zero-valued storage from serialized fields:

- codecdatae and payload_frame_aligned are emitted as explicit one-bit zero values under the traced defaults;
- duplicate flags, priority, and proc_allowed are omitted when payload_frame_aligned is false and sample offset is absent;
- group_id_present is written as zero, so the group_id value is omitted;
- body length and payload follow the configuration, but no body bytes were included or inspected here.

Two washed DEE output inspections show JOC payloads accepted by deployed compatibility while ETSI strict reports payload-11 differences. Those output packages do not bind the loaded DDP module hash to this static image. OpenJOC reports group_id=0 but does not expose whether the presence flag was set. Therefore the static native rule is clear, but its same-run output join and the model's explicit group_id=0 crosswalk remain unbound.

The input-only prefixes differ: 11 bits for the native static Type 11 defaults, 14 bits for the approved compatibility model with explicit group_id=0, and 31 bits for the strict model. These are calculations only; they are not regenerated DEE output or a product-conformance result.
