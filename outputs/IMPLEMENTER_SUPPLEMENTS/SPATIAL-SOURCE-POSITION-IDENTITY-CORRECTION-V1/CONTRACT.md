# Corrected source-position identity receipt

This correction replaces the earlier research-manifest digest for the same scoped observation. In one native invocation, the position builder and candidate installer received the same descriptor object. Row ordinal20 in the 48-row descriptor list has a 56-byte position subrecord beginning at offset24+56*i; its candidate-ID U32 is at descriptor offset40+56*i and equals0. This binds the row subrecord to its candidate-ID field for that call only.

The candidate-ID value does not establish ADM UID, physical host or global source identity. Whether the row passed the candidate gate or was installed was not captured. Row ordinal20 is not called source20. The 48-row native list, 38 physical WAVE tracks and 15 target capacity remain separate domains. The experiment job XML hash differs from the current PreparedJobInput job XML hash, so no current implementation identity join is made.

Latest source manifest SHA-256: 6f6e4eac04d025d5110055951d606fff409fa8d5eab480b28da8fcf405da6aeb. No raw coordinates/audio, addresses, binary names/paths, code, keys or auth material are included. Gate1 remains open.