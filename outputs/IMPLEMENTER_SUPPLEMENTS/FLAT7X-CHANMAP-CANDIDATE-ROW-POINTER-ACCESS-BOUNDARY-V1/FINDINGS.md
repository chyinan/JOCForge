---
status: washed-partial
topic: "D0 chanmap candidate row-pointer access boundary"
spec: "Approved Flat7x profile and ETSI chanmap crosswalk"
confidence: bounded
tags: [JOCForge, Flat7x, chanmap, row-role, provisional]
audience: implementer
---

# D0 chanmap row-pointer boundary

Wash reviewed one sealed mask-parent path. A candidate row pointer is carried through the inspected early branch, but the pointer is not read or written before the subsequent mask-setter call. That call receives the selected parent through a separate argument. The mask path's U16-to-record-to-serializer edge does not identify a row element.

The `0x1A00` ETSI crosswalk identifies Ls/Rs and an Lrs/Rrs pair as coded channel locations. This is a channel-location rule, not a mapping from ADM physical rows to D0 rows or JOC carriers.

No native owner or per-row mapping was added by this slice. Keep the already authorized DirectSpeakers-bed row mapping explicitly provisional and replaceable. Preserve separate typed fields for ADM rows, D0 chanmap locations, Flat7x spatial index, and JOC object count. Do not change the audio algorithm from this boundary package.
