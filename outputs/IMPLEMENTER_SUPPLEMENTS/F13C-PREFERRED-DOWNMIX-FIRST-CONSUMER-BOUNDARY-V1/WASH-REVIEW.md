# Preferred downmix typed-consumer boundary

The schema-validated candidate XML pair varies only `preferred_downmix_mode` (`loro` versus `ltrt`). A separate native `dmixmod` key has a bounded string validator whose allowlist is `not_indicated`, `pro_logic`, `stereo`, and `pro_logic_ii`; no typed owner or alias from the preferred property to that key/consumer is bound. The absence of direct literal xrefs in the bounded query is not a global absence proof.

The numeric mix-level path and generic result-tree path are separate and are not treated as the preferred-mode audio sink. Mode policy, 5.1/JOC audio effect, BSI/metadata effect, UI-only handling, and spatial render effect remain `UNBOUND` or `NOT_COVERED`; candidate pair runtime remains `NOT_RUN`.

No product mode mapping or algorithm change is authorized by this evidence-only boundary.
