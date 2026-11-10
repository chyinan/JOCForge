---
status: user-authorized-partial-delta
topic: Flat7x layout as an independent production profile dimension
spec: ETSI TS 103 420 V1.2.1
tags: [architecture, profile, JOC, flat7x]
audience: implementation
---

# AD-009: Flat7x reconstruction layout is independent

The user has set flat7x output structure as the active target and has authorized partial implementation before every native reverse-engineering edge is closed.

Add `JocReconstructionLayout` as an independent field on the existing immutable resolved-profile value. `Flat7x` maps to the normative JOC header `joc_dmx_config_idx=1`. Keep the current `JocDownmixProfile` (`mode0`, `phase1`, internal `index3`) unchanged and independent. This separates the internal control profile from the JOC decoder reconstruction layout.

The additive Flat7x row reuses the existing 1024 kbps / 5.1 core path and its current input/substream/rate contracts. The existing 5.1 row stays unchanged. There is no new mutable state owner or GTC commit stage because the layout is immutable profile configuration.

This decision approves only the profile model/resolution and 3-bit header-field serialization slice. DEE's exact profile-selector path, full official-demo structure, 7.1 audio payload correctness, and listening acceptance continue as independent follow-up work.
