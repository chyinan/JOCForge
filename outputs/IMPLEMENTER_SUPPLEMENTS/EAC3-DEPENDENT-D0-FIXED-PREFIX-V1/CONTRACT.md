---
status: IMPLEMENTER_READY_LIMITED_DEPENDENT_PREFIX
topic: "Flat7x D0 E-AC-3 dependent syncframe fixed prefix"
spec: "ETSI TS 102 366 V1.4.1, E.1.2.2 and E.1.3.1.3–E.1.3.1.7"
confidence: "ETSI syntax and Wash-approved target row"
tags: [JOCForge, Flat7x, E-AC-3, D0, BSI, prefix]
audience: implementer
---

# Flat7x D0 fixed prefix contract

## Target row

The existing Wash-approved Flat7x target row supplies D0 values: dependent substream, substream ID 0, 48 kHz, six audio blocks, `acmod=5`, `lfeon=0`, `bsid=16`, and `frame_bytes=4096`. ETSI TS 102 366 V1.4.1 defines `frmsiz` as one less than the syncframe size in 16-bit words, so the D0 frame size maps to `frmsiz=2047`. The target D0 map remains `0x1A00`.

## Limited serializer operation

Add an additive helper in the existing `crates/jocforge-domain/src/codec/frame_builder.rs` responsibility. At caller cursor zero, preflight 45 bits and write only this fixed prefix, in order:

| Field | Value | Width |
|---|---:|---:|
| syncword | `0x0B77` | 16 |
| strmtyp | dependent (`1`) | 2 |
| substreamid | `0` | 3 |
| frmsiz | `2047` for 4096 bytes | 11 |
| fscod | `0` (48 kHz) | 2 |
| numblkscod | `3` (six blocks) | 2 |
| acmod | `5` | 3 |
| lfeon | `0` | 1 |
| bsid | `16` | 5 |

The fixed prefix ends at bit 45. The caller owns the following BSI syntax beginning with `dialnorm`; optional fields follow ETSI conditions. For a dependent stream, the later `chanmape=1` and 16-bit `chanmap=0x1A00` are a separate field group, using the already-approved `write_flat7x_d0_chanmap_fields` helper at the correct caller-owned BSI cursor. Do not place that group immediately after bit 45.

The expected 45-bit sequence is `000010110111011101000111111111110011101010000`. With a zero-initialized six-byte buffer, expected bytes are `0b7747ff3a80`; the returned cursor is 45.

## Compatibility and evidence boundary

- Preserve `write_selected_eac3_fixed_prefix` and all existing 5.1 profile/rate behavior unchanged.
- Keep the D0 prefix additive and design-only; do not connect it to the production frame/AU or file-output path. `production_allowed=false` remains.
- This is a structural ETSI-based writer; it contains no audio or JOC algorithm.
- The DEE 6.7 candidate native slice reaches a header serializer and generic bit writer, but its untyped `+0x5c`/14-bit branch is not mapped to the complete D0 target tuple. Do not copy those candidate fields as a serializer recipe.
- The earlier chanmape field-writer supplement carries an `acmod=6` standalone validation vector; that context is not the approved Flat7x target row. This prefix uses the target row's `acmod=5`, `lfeon=0`. The chanmape/map helper itself does not consume acmod.
