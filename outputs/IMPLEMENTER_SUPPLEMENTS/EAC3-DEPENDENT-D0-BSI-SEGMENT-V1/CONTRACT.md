---
status: IMPLEMENTER_READY_LIMITED_D0_BSI_SEGMENT
topic: "Flat7x D0 BSI segment through the custom channel map"
spec: "ETSI TS 102 366 V1.4.1 E.1.2.2 and E.1.3.1.7–E.1.3.1.8"
confidence: "normative BSI syntax plus Wash-approved target row"
tags: [JOCForge, Flat7x, E-AC-3, D0, BSI, chanmap]
audience: implementer
---

# Flat7x D0 BSI segment contract

## Target row

Use the approved D0 profile row: dependent substream (`strmtyp=1`), `substreamid=0`, `acmod=5`, `lfeon=0`, `bsid=16`, frame size 4096 bytes (`frmsiz=2047`), `chanmape=1`, and `chanmap=0x1A00`.

## Limited serializer operation

Add a caller-supplied BSI-segment helper in the existing frame-building responsibility. Starting at bit zero, compose the already-approved 45-bit D0 fixed prefix, followed by:

1. `dialnorm`: caller-supplied U5, valid range 1–31; value 0 is reserved by ETSI;
2. `compre`: one bit derived only from whether the caller supplied `compr`;
3. optional `compr`: caller-supplied U8 when present;
4. for target `acmod=5` (not dual mono) and `strmtyp=1`, emit the separate dependent channel-map group `chanmape=1` then typed U16 `chanmap=0x1A00` using the approved field-pair writer.

Return the cursor immediately after the 16-bit chanmap. The next BSI field `mixmdate` remains caller-owned. Preflight the whole selected segment before the first write: 68 bits without `compr`, 76 bits with `compr`. Reject invalid `dialnorm` and insufficient capacity before mutating bytes or cursor.

The helper serializes fields only. It does not map XML DRC profiles into `dialnorm/compre/compr`; callers supply those values. `compr` is serialized as a caller-supplied field and this helper derives no compression behavior. It does not write `mixmdate`, D0 audio blocks, CRC, EMDF, a complete frame/AU, or a file.

## Compatibility and evidence boundary

- Keep the 5.1 multirate writer and `write_selected_eac3_fixed_prefix` unchanged.
- Keep `chanmape+chanmap` separate from the 45-bit fixed prefix; in the BSI sequence it follows `dialnorm`, `compre`, optional `compr`, and the target's non-dual-mono branch.
- The prior field-pair supplement's acmod=6 vector is a standalone validation context only. The approved D0 target uses `acmod=5`.
- The DEE 6.7 candidate native slice has not typed the observed `+0x5c`/14-bit branch as this target field group; do not copy that path as a recipe.
- This is a standards-based structural continuation, not an audio algorithm. `production_allowed=false` remains.
