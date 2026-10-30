# U04 local source-table slot to category path

Disposition: PARTIAL_STATIC_EVIDENCE_ONLY. This package reports a typed local slot-to-category route. It does not bind that slot to an ADM global source ID and adds no production algorithm or implementation rule.

## Provenance and identity

Source package: spatial-selector-identity-wash-v2.
Source MANIFEST.json SHA-256: 112ecbde696f35a440c769d5d5c315930596b7c9c77ed724a597274628992326.
Source EVIDENCE.json SHA-256: 83825456abb952da307da85e4de0a4d3e867535060c6f3eab3893ba1efc06729.
Base PreparedJobInput XML SHA-256: 20669e276ac7b708b0a03b127d3f37331c8cbc5cfce9a0f4a2498739ecc36cb9.
The attempted execution XML was distinct due to input-kind, schema-version, and local temporary-directory fields; the package reports codec and loudness values were preserved. That attempt exited before measurement and produced no reference output.

The source package manifest, payload hashes, and checker were independently verified. Ordinary Python and python -O checker runs passed. Only this summary and source-binding metadata are included; no raw audio/features, source listing, disassembly, addresses, binaries, license, or key are forwarded. The technical source trace itself was not independently reproduced by Wash; its claims remain attributable to the hash-bound source package.

## Static path reported by the source package

A per-group unsigned source-pointer-table slot index is read by a dispatcher. A type code chooses a category table and the slot selects a pointer from the input source-pointer table; that pointer is routed to the selected category table. A later same-record consumer reads a derived group flag and consumes the routed vectors. The flag is set when at least one item has type 11 or 12 and no item has type 7 or 8; the downstream consumer uses it to choose special category tables for two group positions.

This binds a local slot-to-category-table path only. The package says it scanned decoded operand references for the observed field displacements and found the index reader and flag producer/consumer, but did not observe a direct store to the source-slot-index field. Bulk-copy initialization or an external owner were not excluded. The writer that populates the slot remains unknown.

## Identity boundary

The local slot index is not proven to equal a physical ADM input slot or an ADM global source ID. The prior native wrapper entry observed 48 candidates, 15 targets, and a 24x48 band-major F32 matrix, but the selector gate entry and actual selected-ordinal list were not captured. The prior U04 geometry path separately binds seven local columns to geometry points and mask bits; it does not join them to this source-pointer slot or to the approved F03U 24xN source-weight columns.

Keep these identities separate: ADM global source ID, physical input slot, source-pointer-table slot, feature-matrix column, candidate ordinal, selected ordinal, geometry point, layout-mask bit, U04 column, and source-weight column. Do not join by ordinal, shape, order, event timing, or equal values.

## Current status and next work

No new native observation was captured in this source package; the attempted run failed during audio-component initialization before measurement. Model replay, native gate comparison, Type10, six-plane validation, and Gate1 remain NOT_RUN or NOT_EVALUATED; Gate1 is not closed.

Next: trace the owner or copy that initializes the typed source-pointer-table slot, then find an explicit key or typed crosswalk from that slot to ADM global source ID in the same generation. In parallel, restore the known native launch context to capture gate input, selected ordinals, and their downstream consumer. If runtime startup remains unavailable, continue the static producer/consumer trace without claiming absence from the bounded scan.