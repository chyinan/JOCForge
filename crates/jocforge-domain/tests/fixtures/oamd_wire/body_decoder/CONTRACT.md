# F05O row-read header contract

Status: `DIRECT_DECODER_INPUT_CLOSED`; `PRODUCT_INTEGRATION=NOT_RUN`.

The F05O row-read API decodes `payload_bytes` with the selected coordinate
tables. `row_count` and `leading_lfe` are decoded from the payload header;
they are not caller parameters and must not be supplied separately.

The header is MSB-first:

1. `version`: 2 bits.
2. `count_code`: 5 bits. If it is not 31, `row_count=count_code+1`; if it is
   31, read the following 7 bits and use `row_count=32+extension`.
3. `shortcut`: 1 bit.
4. `leading_lfe`: 1 bit.
5. `protected_sections`: 1 bit.
6. `section_count`: 4 bits (the extended section form remains outside this
   supplement).

When `leading_lfe=1`, the decoded row order is one LFE row followed by
`row_count-1` object rows. When it is zero, all `row_count` rows are object
rows. `grid_route` selects the coordinate table and does not supply either
header value.

The accepted direct decoder shape is conceptually:

```text
decode(payload_bytes, xy_table_63, z_table_16) -> Metadata
```

The returned metadata may expose the decoded header values after parsing. A
caller-provided duplicate value would create an unresolvable conflict and is
outside the contract.

Protected sections, alternate layouts, multi-frame payloads, and product
integration remain outside this supplement.
