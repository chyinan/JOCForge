# Rate-conditioned nested-record consumer

The bounded writer path writes U32 fields into a rate-conditioned nested output record. Its +0x10 U32 is then passed to a generic setter, which stores the value at +0x4 of a separate destination pointer during the same writer-receiver invocation.

The destination pointer has no bound class/type token, constructor, or downstream typed consumer. The projection is static and generic. Its codec-configuration and BSI effect, the nested record type, and the selected-rate setup to the same handle remain UNBOUND. No DEE, media, PCM, payload, key, or license was accessed.
