# Human Cat JOC coefficient-count input lineage

## Washed static path

A per-source tagged configuration record contains a 32-bit value under tag `4`. The value is copied into a per-source mode field, then into encoder state as a 32-bit mode input. In the JOC path, a table lookup maps that mode input to a 16-bit coefficient-slot count stored in encoder state. A previously washed static projection binds the count to both coefficient-delta processing and the serializer's bit-writer length.

This closes the in-module producer to the tagged configuration record. It does not identify who creates that record list or map tag `4` to a public DEE job field, ADM attribute, or user control. The matched run's actual mode value and resulting count were not captured. The JOC audio effect and any relation to the channel-3 PCM residual remain unmeasured.

## Input-only test shape

The proposed one-field mapping vector varies only the tag-4 32-bit value `M`; all matched ADM and job inputs stay fixed. The model path is `M -> 16-bit lookup result N`. `M`, `N`, and runtime audio impact remain unbound until observed. This is not a public setting and should not be exposed as a new user-facing option without a separate schema mapping.

No PCM, encoded payload, private key, license material, native source code, or disassembly is included.
