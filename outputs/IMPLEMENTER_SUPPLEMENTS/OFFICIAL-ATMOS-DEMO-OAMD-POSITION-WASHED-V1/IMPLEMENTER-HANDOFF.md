---
status: wash-reviewed-reference-addendum
topic: "Official-demo OAMD position, room-Z, and object-size observations"
spec: "ETSI TS 103 420 V1.2.1 clauses 4.2.1, 5.2.2, 5.6.1.1"
audience: implementer
production_allowed: false
tags: [JOCForge, Flat7x, OAMD, object-position, reference-sample]
---

# 官方演示参考样本：OAMD 位置与尺寸字段摘要

This sanitized addendum extends `OFFICIAL-ATMOS-DEMO-STRUCTURE-WASHED-V2`. It reports bounded metadata observations from the user-designated 1280 kbps reference sample. The source is hash-bound; provenance is user-designated and not independently authenticated.

## Full-track observations

- 23,033 of 23,033 access units parsed; each has one OAMD element ID 1, 16 OAMD slots, 15 dynamic objects, and one bed/ISF slot.
- Each of the 15 dynamic objects has one explicit absolute position field and one corresponding room-coordinate Z field in every AU: 345,495 explicit position fields total. Differential position fields: zero.
- Every dynamic object is room-anchored throughout the track. Nine objects have Z code 0; six have positive Z code 15, which ETSI TS 103 420 V1.2.1 clause 4.2.1 maps to the room ceiling at +1. No extended-position element ID 5 occurs.
- The separate `b_enable_elevation` zone-constraint flag is enabled on all 345,495 dynamic position updates. Separately, `object_size_idx=0` on all explicit size updates and decoded object-size height is zero across all 345,510 resolved states. Position Z, elevation-zone flag, and object-size height are distinct fields.
- An independent bounded syntax trace matched the full OAMD parser on all 23,033 AUs. Per-AU summaries and hashes were retained in the Wash package; this supplement contains aggregate counts only.

## Relation to the Flat7x target

The reference sample is 1280 kbps (I0=640 + D0=640). The requested 1664 kbps target (I0=640 + D0=1024) remains `NOT_EVALUATED` by this observation. These object-position statistics are a reference fixture for metadata structure and field-domain separation; they do not specify an encoder algorithm or prove target output behavior.

The V2 profile-index/EMDF supplement remains controlling for its own fields: all reference AUs use JOC profile index 1; EMDF Table 56 reports `codecdatae=0` for payload IDs 11/14, while the ETSI table requires 1. Preserve the vendor-compatible observation and ETSI-strict disposition separately.

## Handling boundary

The package contains only this summary, a sanitized aggregate JSON object, source receipts, and a checker. It contains no MKA, packet, OAMD payload, coordinate sample list, PCM, key, license, or research parser source. `production_allowed=false`.