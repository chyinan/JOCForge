# Flat7x D0 skipfld C=511 parser acceptance addendum

This addendum supplements FLAT7X-D0-SKIPFLD-STRUCTURE-V1. It closes the parser-level boundary for one complete 511-byte EMDF carrier in one D0 block-2 skipfld using synthetic input only.

OpenJOC 0.20.1 revision 9654475f10ea9ead020b9ae148dae8b01ab2bef3 parsed a synthetic EMDF container with a four-byte sync/length prefix and 507-byte body (C_total=511). The complete container was supplied as the exact data slice of a single D0 block-2 skipfld with skipl=511. The parser returned Parsed with bytes_consumed=511 and carrier_bytes=511.

The synthetic payload used payload ID 6 with 500 opaque zero-fill bytes. This verifies the container-length and single-field boundary only; it does not validate JOC semantics, licensed payload, audio, or a real output file.

The 9-bit skipl guard still rejects 512 before frame construction. The separate C=542 negative reference uses a 511-byte prefix and is classified malformed/truncated; it is not field fragmentation.

Implementation use: add an exact C=511 boundary test for the field writer/parser seam when available. Keep production_allowed=false and the full Flat7x output gate closed until actual ADM/source/AU owner, JOC payload production, insertion owner, CRC and F15 publication are bound.
