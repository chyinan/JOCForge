---
status: implementer-ready-limited-d0-mixing-and-info-flags
topic: "Flat7x D0 BSI fields after chanmap"
spec: "ETSI TS 102 366 V1.4.1 Annex E, E.1.2.2"
audience: implementer
production_allowed: false
tags: [JOCForge, Flat7x, D0, E-AC-3, BSI, mixing-metadata]
---

# Flat7x D0 mixing metadata and info flags

## Target row

This addendum follows the existing D0 BSI segment helper, which ends after dependent chanmape/chanmap at bit 68 without compr or bit 76 with compr. The target BSI conditions are strmtyp=1, acmod=5, lfeon=0, six blocks. The implementation receives these values as an explicit target-plan context; it does not infer values from ADM row order or DEE runtime state.

## Bounded field sequence

The proposed writer accepts an optional typed mixing-metadata value, six optional block-mix values, and an explicit infomdate boolean. Starting at the parent segment cursor (68 or 76), it emits:

1. mixmdate U1 (derived from presence of the optional mixing metadata).
2. If present: mixmdo U2; ltrtcmixlev and lorocmixlev U3 each; ltrtsurmixlev and lorosurmixlev U3 each. For target acmod=5 all four level fields are present; lfeon=0 omits LFE mix fields. The dependent-stream branch omits independent-only pgmscle/extpgmscle/mixdef fields; acmod=5 omits mono paninfo.
3. frmmixcfginfoe U1 (derived from whether any of the six block values is present); for each of the six blocks, emit blkmixcfginfoe U1 and, when present, blkmixcfginfo U5.
4. infomdate U1.

The helper returns immediately after infomdate; if it is 1, the subsequent informational metadata is a separate caller-owned segment. All numeric mixing and block values are explicit caller inputs; this package does not choose defaults or map XML. It requires complete capacity preflight and rejects invalid values before writing.

## Boundary

This is ETSI-derived syntax for the approved target conditions, not a DEE value capture. It does not implement XML DRC translation, mixdown algorithms, later informational metadata, audio blocks, CRC, EMDF/skipfld, full D0 frame/AU, or production output. It leaves the existing 5.1 E-AC-3 path unchanged. input vectors are structural only, NOT_RUN. production_allowed=false.
