# Direct trim-section codec contract

## Slice identity

`selected-trim-section-codec.v1` is a pure, stateless sub-slice. It covers Type11 trim body encoding/decoding and the minimal DBMD section framing/default-trim projection needed to construct that body. It excludes live queues, parent transactions, raw snapshots, audio, complete DBMD semantics and product integration.

## API

```text
encode_trim_body(trim, row_count)
  -> { body_bytes, valid_bits, padding_bits, group_mode }

decode_trim_body(body_bytes, row_count)
  -> Trim

parse_dbmd_sections(chunk_bytes)
  -> map<U8 section_kind, bytes body>

selected_trim_from_dbmd_sections(sections, row_count)
  -> Trim
```

`Trim` contains `trim_mode: U2`, `group_flags: U1[9]`, `groups: tuple[9][5]`, and `row_flags: U1[row_count]`. Default group is `[16,16,16,0,0]`.

## Trim body bits

MSB first:

1. trim mode U2;
2. reserved U2 = 0;
3. group mode U2.

Group mode is derived:

- 0: every group flag is one; no group payload;
- 1: all flags zero and all groups default; no group payload;
- 2: explicit nine-group sequence.

Mode 2 writes each group flag. Flag one skips the group. Flag zero writes a one-bit whole-default sentinel. Sentinel one skips the group. Sentinel zero writes a U5 field mask. Selected fields 0..2 are U4 `0..15`; unchanged default value 16 is represented by a clear mask bit, never by U4. Selected balance fields 3..4 encode sign U1 and `abs(value)-1` U4 for values `[-16,-1]∪[1,16]`.

After groups, write `row_presence=any(row_flags)`. If one, write exactly `row_count` U1 flags. If zero, no row bits follow and decoded flags are zero.

The body returns its exact valid bit count. Standalone bytes append zero bits to the next byte boundary. Decoder consumes the semantic fields and requires every remaining padding bit to be zero. No byte terminator is part of the trim body.

## Minimal DBMD framing

The framing-only parser starts after a four-byte little-endian header. Each section is:

```text
kind U8 || body_length U16LE || body bytes || checksum U8
checksum = -(body_length + sum(body bytes)) mod 256
```

Kind zero terminates parsing; every remaining byte must be zero. Duplicate kinds, truncated header/payload, checksum mismatch and nonzero terminator tail are rejected.

Selected default trim projection requires section 9 body length 248 and section 10 length at least 142 with magic `0xf8726fbd`. Warm mode is section9[152]&7 and must be 0..3. The nine section10 records provide flags. When a flag is zero, nonzero gain/balance values are outside this sub-slice and return `DBMD_NONDEFAULT_TRIM`.

## Errors

- `TRIM_SHAPE`, `TRIM_DOMAIN`, `TRIM_U4`, `TRIM_SIGNED_BALANCE`
- `TRIM_TRUNCATED`, `TRIM_RESERVED`, `TRIM_GROUP_MODE`, `TRIM_NONZERO_PADDING`
- `DBMD_HEADER`, `DBMD_TRUNCATED_HEADER`, `DBMD_TRUNCATED_PAYLOAD`
- `DBMD_CHECKSUM`, `DBMD_DUPLICATE_SECTION`, `DBMD_TERMINATOR_TAIL`
- `DBMD_REQUIRED_SECTIONS`, `DBMD_SUPPLEMENTAL`, `DBMD_WARM_MODE`, `DBMD_NONDEFAULT_TRIM`

Failures return no partial Trim or bytes. This pure slice owns no state and has no commit/rollback behavior.

## Evidence and boundaries

The independent implementation matches frozen `type11_control_wire_core.py` for five success cases and frozen `type11_dbmd_trim_core.py` for the selected default DBMD projection. Expected bytes were compared after computation.

Specification/model facts: bit order, group modes, default sentinel, field mask, U4 and signed balance encodings, row-presence rule, zero padding, DBMD framing/checksum/duplicate/terminator rules and selected default projection.

`BLOCKED` outside this slice: nondefault DBMD gain/balance quantization, other DBMD section semantics, alternate Type11 profiles, live queue, parent transaction and native/product integration.

`NOT_RUN`: full DBMD parser, real WAV/ADM, codec/backend/writer, queue/publication and private-key paths.
