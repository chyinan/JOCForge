# F09-D45 and F12 current Rust coverage boundary

## Result

A hash-bound static coverage audit found the approved F12 quantizer and BAP grouping/packing paths in the current Rust source. Within the bounded `jocforge-domain/src`, `jocforge-application/src`, and `jocforge-cli/src` search, it found no direct F09-D45 selector-cell → symbol-table → 4/7-bit accumulator path.

The separate Rust `db_selector_from_means` path produces a dB-per-bit selector used in a 2-bit BA tuple. The washed DEE D45 selector is a distinct U16 field whose symbols enter 4-bit and 7-bit writes. Similar widths or U16 values do not establish identity.

This is a source-coverage finding, not a cause for the attempt-10 whole-file hash mismatch. The D45 semantic field and absolute stream bit position remain `UNBOUND`; attempt-10 executable-to-source-tree identity is also `UNBOUND`.

## Scope

The scan covered only the current Rust domain, application, and CLI source directories with the listed D45 markers. It does not prove no differently named path exists outside that search. No tests, encoder, process, prototype Python source, output payload, or comparator payload were read or run in this audit.
