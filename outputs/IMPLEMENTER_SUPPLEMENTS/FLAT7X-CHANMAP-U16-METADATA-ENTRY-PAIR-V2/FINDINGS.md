---
status: washed-partial
topic: "D0 chanmap candidate U16 metadata-entry pair"
spec: "Approved Flat7x profile and Wash static boundary"
confidence: bounded
tags: [JOCForge, Flat7x, chanmap, metadata-entry, U16]
audience: implementer
---

# D0 chanmap candidate U16 metadata-entry pair

Two consecutive writes are now observed through the same candidate metadata-row pointer: a U16 at offset `0`, followed on a separately traced successor by a U16 at offset `+2`. The second value comes from a composite memory read and is then stored at the `+2` pointer expression. This establishes a two-byte adjacent destination pattern only.

The candidate pointer's typed owner, the values' semantic labels, table cardinality, row/channel role, and join to D0 `chanmap=0x1A00` remain unbound. The ETSI mapping of `0x1A00` to Ls/Rs plus the Lrs/Rrs pair is a distinct channel-location rule and does not name these two entries.

Keep this candidate metadata table separate from the provisional PCM row mapping, D0 coded channel locations, and JOC reconstruction carriers. The existing user-authorized DirectSpeakers PCM map remains provisional and replaceable; this package defines no new audio algorithm or role mapping.
