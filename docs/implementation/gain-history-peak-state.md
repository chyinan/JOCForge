# Gain peak, history, and reuse state

This F09G direct slice covers selected pass-0, summary-gate-1, format-7 gain processing for six native-order channels over six blocks. It owns only the typed selected fields: six previous peaks, a six-by-six summary matrix, six cumulative Q15 maxima, and six-by-five full-band gain output. LFE contributes to peak/summary/Q15 statistics but never emits a gain code.

Peak reduction profile is an explicit input. The aligned four-lane path preserves lane and horizontal reduction operand order; the unaligned scalar path uses `MAX(sample, previous)` so ties retain the earlier high operand. Both MIN/MAX rules select the second operand on equality, including signed-zero cases. Current peak is `R32(2*peak)`; summary uses `MAX(previous,current)` with current as the second operand; Q15 uses round-to-nearest-even, signed saturation, and cumulative signed maximum.

For channels 0..4, the helper applies mask-specific CENTER/SIDE additions in the prescribed order, clips with the same second-operand tie rule, derives headroom from F32 bits, and writes a gain code. It does not apply filter/energy auxiliary effects. Reuse-min is separate: each nonzero block strategy starts an anchor; zero-strategy gains merge into that anchor with strict signed `<`, then the segment minimum is propagated.

Seventeen history cases and three reuse cases are generated from input-only rules. Tests recompute 165,888 sample words, 648 peak transitions, 540 gains and 90 reuse values. Derived lane traces are compared as static-rule calculations, not native register captures; final owner fields are compared against the package results. Runtime receipts and static-derived traces keep their separate evidence grades.

Filter-state numerical effects, reset/fatal/EOF, absolute ADM sample mapping, external delayed-six input production, and whole F09/audio integration remain open. This helper does not reuse primary/companion FFT history as gain state.

Package manifest SHA-256: `32a5d555edbb83123e17a51843ce2d3d905944815eee3d70924854a23ddbb419`.

Independent-check SHA-256: `dc49214b761ead381a230f9a10a7ec44ffe95691391230105fe1b520c44333c2`.
