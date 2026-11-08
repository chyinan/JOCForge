# Data-rate type-1 encoder boundary

The six XML tokens map statically through enum codes 0..5 to core integer values 384, 448, 576, 640, 768, and 1024; the common builder stores them at field ID 3 and passes a rate-bearing table through the generic type-1 serializer slot +0x28. This binds a static parameter path only.

The type-1 callback target and the field-3 join to a named E-AC-3 carrier, frame-size word, or emitted bitrate remain UNBOUND. The static callback cell being zero does not establish its runtime value. Header readers, existing bse_frmsiz validators, and the separate type-2 DRC writer are not configured-rate consumers.

The numeric kbps interpretation is an inference from a separate 448-kbps diagnostic; no unit annotation or output rate was recovered. A 1024-vs-768 one-field contrast remains PROPOSED_NOT_RUN, with no expected frame-size result. No encoder run or payload inspection occurred.
