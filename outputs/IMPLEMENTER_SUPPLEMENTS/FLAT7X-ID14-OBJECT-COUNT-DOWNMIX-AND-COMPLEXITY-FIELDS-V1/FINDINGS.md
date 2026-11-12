---
status: washed-partial
topic: "Flat7x ID14 object count and independent profile dimensions"
spec: "ETSI TS 103 420; approved Flat7x target profile"
confidence: medium
tags: [JOCForge, Flat7x, ID14, object-count, downmix-index, complexity]
audience: implementer
---

# Flat7x ID14 and complexity field contract

## Approved target values

- The active target profile remains I0 640 kbps followed by D0 1024 kbps, aggregate AU 1664 kbps.
- Flat7x uses JOC spatial index 1, 15 JOC objects, and target `complexity_index_type_a` 16.
- D0 `chanmap=0x1A00` remains its own channel-location field.

## Washed serializer rules

In the selected DEE 6.7 static ID14 path, the object-count header value is six bits and is formed as `object_count - 1`. For count 15, serialize value 14 in six bits. This native rule is not joined to a particular runtime invocation.

A separate three-bit `downmix_index` value 1 selects coefficient processing width 7 on the inspected path. Keep `downmix_index=1` distinct from Flat7x `joc_spatial_index=1`; the equal integer values identify different dimensions. The coefficient-generation algorithm is outside this package.

## Complexity separation

`complexity_index_type_a=16` is an explicit target-profile value. A separate native F12 candidate path reads a configuration scalar and serializes one byte, but this slice does not bind that scalar to the ID14 count, the Flat7x profile, or a same-generation run. Do not derive native complexity from object count; keep the two typed fields independent.

## Required model shape

Keep `joc_spatial_index`, `downmix_index`, `joc_object_count`, `complexity_index_type_a`, ADM physical-row count, and D0 `chanmap` as separate fields. For this target, the known vector is:

```text
joc_spatial_index=1
downmix_index=1
downmix_coefficient_width=7
joc_object_count=15
id14_object_count_bits=14 (6 bits)
complexity_index_type_a=16 (8-bit field)
d0_chanmap=0x1A00
```

The profile-specific assumptions remain replaceable. This package covers header field serialization and independent dimensions; it does not define D0 channel-role ownership, the seven-carrier audio algorithm, native F12 owner/generation joins, or runtime/output equivalence.
