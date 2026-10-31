# Wash review: ADM reader factory boundary

Evidence-only supplement from a static Ghidra package. Source manifest SHA-256 is recorded in MANIFEST.json.

- R138 is hash-bound to input type audio.adm, route encode_to_atmos_ddp, and embedded_timecode; its timecode rate is not_indicated.
- A host WAVE adapter Run path directly calls a shared source-time metadata normalizer. The edge is local to that adapter class; its applicability to R138 audio.adm is not bound.
- ADM/BWF storage/parser components exist in the storage library, but their presence does not prove R138 factory selection or a host-to-parser typed transfer.
- The concrete R138 reader factory, provider-to-parser edge, parsed descriptor-to-Atmos/codec handoff, and reader output's first processing consumer remain UNBOUND. No AtmosMezz dispatch was established.
- This was static Ghidra analysis only: no authorized audio sample used, no native run, and no PCM/bitstream read or saved.

This package helps avoid treating a generic WAVE metadata call or library component as the selected ADM path. It does not close ADM reader selection or the audio handoff, and is not an implementation-ready algorithm specification.
