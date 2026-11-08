# RF film_light static setter and first-consumer boundary

Reference-only Wash supplement. It records one static schema-default path for `drc:rf_mode_drc_profile`; it does not prove the explicit XML value reaches the same StringType value slot.

The schema-default path stores `film_light` in the StringType value member at `+0x08` on the static `Priv root+0x750` layout. The per-frame getter reads that value, and the RF-local map yields raw 32-bit candidate `0` at the per-frame record candidate `+0xa4`.

The XML lookup node is separately copied to `+0x28`. No same-owner write from that node to `+0x08`, selected provider instance, or runtime generation is bound. Raw code `0` has no assigned DRC branch, BSI, or audio meaning. Runtime and audio effect are `NOT_RUN`; `IMPLEMENTATION_READY=NO`.
