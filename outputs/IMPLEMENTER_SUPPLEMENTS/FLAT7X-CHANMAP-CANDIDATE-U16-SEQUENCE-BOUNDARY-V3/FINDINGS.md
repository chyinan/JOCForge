---
status: washed-partial
topic: "Flat7x candidate U16 sequence boundary"
spec: "Approved Flat7x field separation and Wash static lineage"
confidence: bounded
tags: [JOCForge, Flat7x, U16, metadata-sequence, boundary]
audience: implementer
---

# Candidate U16 sequence boundary

The selected native path contains six consecutive U16 destination writes at two-byte steps, beginning after the already observed `R15+2` destination. Values are loaded through a composite address before each store. The base is an opaque live-in; the imported Ghidra project has no direct call reference binding it to a typed owner.

This observes a local write sequence only. Its total structure, field labels, semantic role, selected profile, and relation to D0 `chanmap=0x1A00` remain unbound. The local count of six is not a synonym for ADM physical rows, E-AC-3 blocks, coded channel locations, or JOC carriers.

Keep the user-authorized PCM row-role map as the current provisional seam. Do not derive its contents from this U16 sequence and do not change the audio algorithm from this boundary package.
