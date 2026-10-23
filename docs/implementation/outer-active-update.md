# F03S.4 active energy update

This direct function consumes explicit F32 context, response, seed, and caller-supplied correction arrays. It computes unscaled `y=A×x`, the initial-scaled seed/y, `v=A×correction`, the ordered energy terms and discriminant, then the selected step and output. Every multiply/add/subtract/divide/square-root follows the specified separate R32 order. A negative discriminant rejects; it is not clamped to zero.

The implementation does not generate correction, run the NNLS solver, select the active branch, or apply F03S.5 floor/prune. The source has one active-update success vector and a separate explicit negative-discriminant rejection probe; malformed and non-finite inputs are rejected locally.
