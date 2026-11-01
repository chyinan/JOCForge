# Wash review: ADM/BWF factory registration boundary

Evidence-only supplement from a static Ghidra package.

- Static host registration for the audio_input category is closed: a category getter returns an ordered function-pointer list and count, and the host appends the entries into its registry.
- This does not identify an ADM/BWF reader factory entry or bind any registry ordinal to the R138 audio.adm input.
- ADM/BWF parser components exist in DACT, but the DACT getter-to-audio_input registry transfer is NOT_OBSERVED. A built-in host reader remains possible.
- A selected reader Run call is downstream of factory selection. To identify the reader, join the registry entry, reader object/vtable/Run target, typed audio.adm owner, and parent-generation token within one task.
- The first reader-output descriptor/frame handoff into Atmos/codec remains UNBOUND. The package's one-shot observation plan reads no payload; no authorized sample or native job was used.

This closes the generic category registration mechanism only. It does not close R138 reader selection or the audio handoff and is not an implementation-ready algorithm specification.
