# Wash review: ADM input-object dispatch boundary

Evidence-only supplement from a static Ghidra package. Source manifest SHA-256 is recorded in MANIFEST.json.

- R138 is identified by a hash-bound config as audio.adm on the encode_to_atmos_ddp route with embedded_timecode.
- The host has an input-object lifecycle wrapper for the selected reader and a target arranger. The generic WAVE adapter Run-to-shared-time-normalizer edge is not bound to R138 audio.adm.
- Static ADM/BWF storage/parser components do not prove the concrete R138 reader factory or its typed transfer.
- The package proposes a minimal discriminator immediately before the selected reader Run call: correlate wrapper owner, reader object/vtable/Run target, typed audio.adm input, task-generation token, thread, and caller stack from the same owner. It reads no payload. This is an observation plan, not an observed runtime result.
- Concrete R138 factory, reader-output descriptor/metadata/frame handoff, and first Atmos/codec consumer remain UNBOUND. No native job or authorized audio sample was used in this static package.

This narrows the next observation needed; it does not establish reader identity, audio payload flow, or an implementation algorithm.
