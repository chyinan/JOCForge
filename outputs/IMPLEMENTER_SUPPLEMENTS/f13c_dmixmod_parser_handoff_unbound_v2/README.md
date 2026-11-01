# F13C dmixmod parser handoff boundary

Ghidra 12.2 static evidence from the existing imported DEE image binds the configuration parser to five separate keys: four Lo/Ro or Lt/Rt mix-level fields and dmixmod. The enum field uses a distinct converter and internal slot. The absent-field path skips value parsing; the value initialized or used afterward is not established.

The parser-result path reaches a generic configuration handoff, but the targeted handoff body did not show field-specific reads. No direct DME preferred_downmix_mode to native dmixmod edge was established, and no final F13C writer/consumer branch was traced. Do not infer conversion from similar names.

Wash compared the Ghidra projects recorded image SHA with a fresh SHA-256 of the current installed DEE executable; they match. No new encode, PCM read, payload read, or key access occurred.
