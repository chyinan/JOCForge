# Quantized Huffman body writer

This direct slice maps already-quantized coefficient and selected-channel inputs to the F05Q Huffman body bitstream. It uses the six table values authorized by the user for public-repository use; third-party license clearance remains `UNKNOWN` and is not represented as ETSI written permission.

The local writer covers mode-to-table selection, full-mode per-point/channel resets, sparse position-table selection, point-local sparse previous values, MSB-first codeword output, zero padding, table/symbol validation, and the local 4,344-bit scratch and final-byte capacity checks. The input begins after fit and quantization. It contains no audio-to-coefficient processing, retry loop, ID14 section header, or parent transaction commit.

## Section-framing boundary

A separate selected ID14 header/body writer now covers the washed header fields, descriptor expansion, body splice, extension trailer, padding, status/return distinction, output-length behavior, and bounded capacity vectors. It is tracked independently at `F05Q/id14-header-body-writer`; its contract does not close the upstream producer mapping, kind/mode semantics, full parent rollback, or the whole ID14 transaction. Do not describe the two direct slices together as a complete F05Q section producer.

## Verification boundary

The local tests match the washed table and consumer vectors, including exact body bytes, valid-bit counts, padding, table selection, and reset events. ID14 header/framing vectors are covered separately by `crates/jocforge-domain/tests/id14_header_framing.rs`. This is implementer-code verification against the washed packages, not OpenJOC archive-import replay or native/runtime replay; those remain `NOT_RUN`. Whole F05Q, full JOC integration, and whole-product readiness remain deferred. The user authorized public-repository use of the table data; this is not represented as ETSI written permission, and third-party license clearance remains `UNKNOWN`.
