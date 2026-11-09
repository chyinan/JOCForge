# Shared verified conclusions

Only conclusions supported by the sealed or approved Wash packages below are listed here.

- The approved 7.1 flat7x target separates I0 at 640 kbps from D0 at 1024 kbps. The 1664 kbps total is a two-frame AU allocation, not one 1664 kbps E-AC-3 frame. Production remains disabled while the GTC role projection, D0 algorithm, and all-AU EMDF fit remain open. Evidence: `JOCFORGE-ATMOS71-FLAT7X-1664-PROFILE-CONTRACT-V1`.
- The target AU contract orders separate I0 and D0 inputs, requires one source generation, and rejects I0-only partial publication. This is a target requirement, not proof of legacy durable atomic publication. Evidence: `JOCFORGE-I0-D0-AU-COMPOSITION-BOUNDARY-V1`.
- The rate/frame-size supplement records target arithmetic of 2560-byte I0 and 4096-byte D0 frames, plus conditional setup candidates. The setup-to-writer join, D0 `frmsiz` producer, BSI bitrate field, and actual output remain `UNBOUND/NOT_RUN`. Evidence: `DEE-ATMOS71-I0-D0-RATE-FRMSIZ-BOUNDARY-PARTIAL-V1`.
- E-AC-3 chanmap U16 numeric serialization uses emitted slot `15 - numeric_bit_index`; the `0x1A00` field has emitted slots `[3,4,6]`. This does not derive semantic speaker labels or bind ADM source rows to D0 roles. Native execution is `NOT_RUN`. Evidence: `JOCFORGE-EAC3-CHANMAP-U16-SERIALIZATION-V1`.
- The older partial transaction package observes separate static writer boundaries but does not identify them as I0/D0 or prove AU order, generation, or atomic commit. Evidence: `DEE-ATMOS71-I0-D0-AU-TRANSACTION-BOUNDARY-PARTIAL-V1`.
- Old main: reference-only; it is not an implementation source.
