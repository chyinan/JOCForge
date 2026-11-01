# F09.4 no-row behavior: cold versus synthetic warm force masks

This package combines the Ghidra static no-row return/caller facts with input-only enumeration; it does not claim a native no-row observation.

On a cold owner with force-D45 history zero, marker[0] fixed to one and all 32 combinations of the five later marker-after bits produce 32 distinct U2 rows, all present in the verified Table E.1.9 set. Thus the cold/no-force path has no no-row case under the traced update order.

With arbitrary six-cell binary force-D45 masks, synthetic enumeration yields 1,495 nonmatching rows out of 2,048 marker/mask combinations. The smallest displayed witness changes [1,0,0,0,0,0] to [3,0,0,0,0,0]. This shows the unchecked branch is reachable in the synthetic model, not that DEE can produce that mask; the later nonzero force-history writer remains UNBOUND.

Static caller behavior is concerning: on no match the selector returns 0xC without writing the short output slot; the caller ignores the return and sets ready. A fresh slot remains zero, while a warm slot may retain the previous code; the downstream stuffer serializes the provided short without validation. No runtime occurrence was observed. Do not make this a general native conclusion. For implementation, reject an unmatched row unless a valid source-state contract proves it cannot occur.
