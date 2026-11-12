---
status: washed-partial
topic: "Flat7x D0 channel-map metadata-row boundary"
spec: "Approved Flat7x profile and Wash static boundary"
confidence: bounded
tags: [JOCForge, Flat7x, chanmap, metadata-row, U16]
audience: implementer
---

# D0 channel-map metadata-row boundary

Wash has now observed a U16 write through a candidate row-pointer alias after a mask-setter returns. The written value is assembled from three owner-relative U8 inputs. The input fields' semantic labels and the U16 row's role remain unbound; the U16 value is not joined to the `0x1A00` chanmap crosswalk.

Keep this metadata-row pointer separate from the Flat7x PCM-row projection. Keep D0 chanmap locations separate from both. The already authorized DirectSpeakers PCM role map remains provisional and replaceable; this package adds no channel-role mapping and changes no audio algorithm.
