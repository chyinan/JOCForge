# Single-frame OAMD body assembler

## Identity and API

`single-frame-oamd-body-assembler.v1` is pure and stateless.

```text
assemble_body(Metadata { rows, timing, trim })
  -> { bytes, valid_bits, padding_bits, row_count, leading_lfe,
       section_count=2, section descriptors }

decode_body(bytes, xy_table_63, z_table_16)
  -> { Metadata, row_count, leading_lfe, section_count, section descriptors }
```

`Trim` is already parsed caller input. This slice does not parse DBMD.

Rows contain kind, mute, gain, spread, XYZ Q15, component, z-valid and diffuse. Timing contains base, position, duration and precision.

## Outer header and order

MSB first:

1. Type11 version U2 = 0;
2. row count: five-bit `min(count-1,31)` and, for count≥32, seven-bit `count-32`; accepted count 1..159;
3. shortcut U1 = 1;
4. leading-LFE U1, derived only from row0 kind;
5. protection U1 = 0;
6. section count U4 = 2.

Exactly two sections follow in this order:

1. kind 1 rowdata section;
2. kind 2 trim section.

The only supported row layout is optional one leading LFE followed by object rows. Any LFE elsewhere is rejected.

## Section framing and padding

Each section is:

```text
kind U4
escaped(payload_bytes - 1), bijective base-16 digits U4 each followed by continuation U1
payload bytes
```

Section payload begins with control U1 = 0, followed by its semantic body. Each section payload is independently padded with zero bits to a whole byte before its escaped byte length is written. Those inner zero bits are part of the section payload bytes.

After both sections, the outer body is zero padded to a whole byte. `valid_bits` is the number of outer bits before this final padding; `padding_bits = bytes*8-valid_bits`. Decoder rejects nonzero section or outer padding, duplicate/missing/reordered kinds, truncated sections and unterminated escaped lengths.

The rowdata section contains timing then rows. The trim section uses the previously delivered trim-body contract.

Final assembled body capacity is 160 bytes.

## Timing

- base: 0 shortcut, table values 8/16/18/24, or explicit five-bit value;
- exactly one frame;
- position U6;
- duration shortcut 0/512/1536, table of sixteen values including 2048, or explicit U11;
- precision 32 shortcut or explicit U5 0..31.

## Errors

- `ROW_COUNT`, `LEADING_LFE`, `ROW_DOMAIN`, `ROW_XYZ`, `ROW_GAIN`, `ROW_SPREAD`, row-extension errors;
- timing base/domain/duration/reserved/multiframe errors;
- trim shape/domain/U4/signed-balance errors;
- `SECTION_COUNT`, `SECTION_KIND`, `DUPLICATE_SECTION`, `SECTION_MISSING`;
- `TRUNCATED`, `SECTION_OVERFLOW`, `ESCAPED_OVERFLOW`, `ESCAPED_UNTERMINATED`;
- `NONZERO_PADDING`, `UNCONSUMED_FIELDS`, `BODY_CAPACITY`.

Failure returns no partial body or decoded Metadata.

## Verification and boundaries

Four input-only success vectors cover seven-row leading-LFE composition with trim mode 0, trim mode 2 with default sentinel/mask/signed balance, one-group escaped length, two-group escaped length, timing table/explicit forms and multiple padding lengths.

All four assembled byte strings and independent decode projections match frozen `type11_control_wire_core.py`; expected values were not computation seeds.

This composition is completely specified by the frozen model, so no DLL lookup was needed for this slice. Native/product execution remains outside the claim.

`NOT_RUN`: F05L live owner, ID11 queue, DBMD parsing, parent/cross-root transaction, raw snapshot, real ADM, codec/backend/writer and product integration.
