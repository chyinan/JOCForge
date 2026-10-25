# Control52 selector-zero condition

This supplement records a conditional static path only; it does not claim the current ADM input takes that path.

For a context whose source mode is not 1, the selector table is initialized to zero and populated from in-range channel records. Supported categories produce nonzero selector values. If a category is unsupported, the mapper returns a stop code; the builder stops at that input row but returns success, leaves the in-range selector slot zero, and does not compact the input count. The caller does not reject that successful result. The selected callback reaches the Control52 row writer only when the context writer-enable flag is clear; a zero selector then takes a no-write branch, preserving the prior row's semantic content.

The constructor interval identity used for the context-mode fields is corrected to the image-bound region hash in `EVIDENCE.json`; the older unlocated digest is withdrawn for that interval. The static conditions prove only that selector-zero is conditionally reachable when both `source_mode != 1` and an unsupported in-range category are present. Whether the selected ADM graph satisfies either condition is `NOT_OBSERVED`.

The formatter output and selected handler gate table remain different owners. No same-pointer or same-generation join from formatter output to the selected gate is established. Do not infer a live gate value from this branch rule or from matching field offsets.
