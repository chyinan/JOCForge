---
status: washed-partial
topic: "Flat7x per-span FileWriter sink and pair staging"
spec: "Approved I0+D0 AU composition contract and Wash FileWriter boundary"
confidence: high for selected sink call; native stream/AU joins unbound
tags: [JOCForge, Flat7x, FileWriter, AU, staging]
audience: implementer
---

# Flat7x per-span FileWriter sink contract

## Selected native sink path

On one selected repeated writer path, the parent holder reaches a provider call that returns a byte-span start/end pair. The caller computes `length = end - start` and passes the span, length, and a sequential write ordinal to a FileWriter routine. For nonzero length, the routine calls `fwrite(buffer, 1, length, FILE*)`; a short-write error is checked after the call.

## Flat7x implementation contract

Keep the user target as one AU with I0 640 kbps followed by D0 1024 kbps. Use a pair-owned staging candidate: validate and append the I0 span, then the D0 span, and expose the completed AU only after both spans pass. A write failure retires the candidate without publishing a partial I0-only or D0-only AU. The existing 5.1 output route remains unchanged.

The native observation is per-span. It does not identify this repeated writer's stream role, AU identity, source generation, pair transaction, file identity, or durable-storage semantics. The atomic pair behavior comes from the approved Flat7x target/transaction contract and JOCForge-owned staging; it is not inferred from the native `fwrite` path.
