# OAMD processing

Last verified: 2026-10-20

## Purpose

Keep washed OAMD wire operations on resolved values in the pure domain layer. This subdomain does not admit raw application input or coordinate the live metadata lifecycle.

## Contracts

- `body_assembler::assemble_body` consumes resolved rows, timing, and trim, and emits exactly one rowdata section followed by one Type11 trim section.
- `body_assembler::decode_body` decodes that bounded two-section body and requires caller-supplied PG06 coordinate tables.
- Body assembly is stateless and capped at 160 bytes. It does not parse DBMD input or install metadata into an owner.
- Rowdata, body-reader, and trim-section APIs remain independently scoped direct slices; their implementation map records the exact claims and deferred work.

## Dependencies

- Uses only sibling OAMD value and wire modules: `rowdata`, `body_reader`, and `trim_section`.
- Remains inside `jocforge-domain`; provider files, environment access, callback clocks, and parent transactions belong outside this module.

## Key decisions

- Keep the single-frame body assembler separate from F05L live-owner and ID11 queue work; those remain deferred.
- Keep full F05O and end-to-end product acceptance distinct from this locally validated direct slice.

## Invariants

- The assembler accepts already-resolved values and never mutates committed programme state.
- The body contains exactly two sections in rowdata-then-trim order.
- Local vector validation is not a claim of native, W04, or end-to-end codec conformance.

## Key files

- `body_assembler.rs` - bounded single-frame section assembly and decoding.
- `rowdata.rs` - rowdata value encoding.
- `body_reader.rs` - rowdata body decoding.
- `trim_section.rs` - Type11 trim section values and framing.
- `../../../../../docs/implementation/IMPLEMENTATION-MAP.json` - exact slice scope and validation claim.
