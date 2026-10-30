# Source-weight local-ordinal path

## Bounded positive finding

The hash-bound static call path shows the selected callback materializer iterating a count-indexed source-buffer pointer vector and copying each candidate's feature data to the destination buffer at the same local ordinal. The selector emits local candidate ordinals; the native output reassembly iterates those ordinals, inserts default rows at matching local positions where needed, and the traced downstream score calls consume matrix views/dimensions without an explicit global candidate-ID argument.

The source-weight caller passes a matrix view and source count. Run 120 binds the caller's matrix object to the consumer for a 24-band/25-source matrix and 600 in-helper `powf` calls. Thus the traced selection/reassembly/source-weight path is position-based on local ordinals and matrix columns.

## Identity limit

The 32-bit per-candidate descriptor is constructed from five scalar metadata fields and parsed back into five fields; it is a packed metadata descriptor, not a proven global candidate ID. The frozen numeric model's `id` field is a vector-case label and exposes no ID-to-column API. Run 120 did not capture candidate lists/global IDs; run 121 did not reach the native callback. The outer ADM graph/export identity path was not traced. Keep global candidate ID → matrix column → weight output `UNBOUND` outside the bounded local-ordinal path; do not infer it from order, counts, descriptor values, or hashes.
