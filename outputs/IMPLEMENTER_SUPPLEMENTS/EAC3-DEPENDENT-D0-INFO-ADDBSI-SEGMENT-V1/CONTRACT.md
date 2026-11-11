---
status: implementer-ready-limited-d0-info-addbsi-suffix
topic: "Flat7x D0 informational metadata and addbsi suffix"
spec: "ETSI TS 102 366 V1.4.1 Annex E, E.1.2.2 and E.1.2.0"
audience: implementer
production_allowed: false
tags: [JOCForge, Flat7x, D0, E-AC-3, BSI, addbsi]
---

# Flat7x D0 information and addbsi suffix

## Bounded scope

This supplement starts immediately after the infomdate bit emitted by the parent D0 mixing/infomdate helper. It serializes the infomdate-selected information fields and the remaining conversion/addbsi grammar for the approved target conditions: strmtyp=1, acmod=5, lfeon=0, fscod=0, six blocks.

The caller passes the same infomdate presence boolean used by the parent and either supplies the corresponding information fields or omits them. The helper rejects a disagreement before writing. All values are caller inputs; the helper does not infer XML, DME, or DEE values.

## ETSI field order after infomdate

If infomdate is 1, write bsmod U3, copyrightb U1, origbs U1, audprodie U1 and, if present, mixlevel U5 / roomtyp U2 / convtyp U1. For target acmod=5 the dsurmod/dheadphonmod and dsurexmod branches are absent. Because target fscod=0, sourcefscod U1 follows the information fields.

For target strmtyp=1, the independent-stream convsync branch and strmtyp=2 conversion branch are absent. Then write addbsie U1. If addbsi bytes are supplied, require 1..64 bytes, write addbsil U6 as byte_count-1, followed by the caller's bytes. The suffix ends immediately after those bytes (or after addbsie=0).

## Validation and limits

Validate target context, infomdate/input agreement, all field widths, optional addbsi length, and capacity for the entire suffix before the first write. The vector containing bytes 0x12 0x34 is a synthetic serializer input, not an official payload or JOC recipe. The official reference reports a 2-byte addbsi field for its 1,280-kbps sample; the 1,664-kbps target addbsi content remains caller-owned and unobserved.

This helper writes only the suffix after infomdate. It does not create F13/EMDF data, interpret JOC addbsi, calculate CRC, create audio blocks, assemble a frame/AU, or enable production. The existing 5.1 writer remains unchanged. production_allowed=false.
