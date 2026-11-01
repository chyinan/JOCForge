# Wash review: Run-100 strict EMDF deviations

The hash-bound OpenJOC metadata reports JOC present and successfully parsed Addbsi signaling across all 4,247 AUs. ETSI strict validation fails on seven EMDF flag deviations repeated from AU 0; deployed-compatibility accepts that explicit set. These facts are independent: the strict failure does not mean JOC is absent, and the compatibility pass does not prove general decoder interoperability.

The deviations are:
- Payload 11: codecdatae 0 instead of 1; payload_frame_aligned 0 instead of 1; create_duplicate, remove_duplicate, priority, and proc_allowed absent instead of 0.
- Payload 14: codecdatae 0 instead of 1.

This is a concrete interoperability risk to assess for the supported output profile. Run-100 encoder completion remains unconfirmed and render_verified is false. The package is metadata-only; no PCM or payload bytes are included. R138 remains a separate embedded_timecode lineage.
