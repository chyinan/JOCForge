# Long-block LFE transform to seven raw exponents

This direct slice implements the selected finite 256-word long-block LFE path. It shares only the washed fold/pre-rotation prefix with the full-band long transform. The following transform is distinct: sixteen gathered eight-complex groups, a dedicated FFT8 operation sequence, a recurrence over the groups, and the selected seven-bin post-rotation. The result's primary and companion seven-word rows are passed directly to the selected LFE coefficient-pair raw-exponent helper.

Each call receives separate caller-owned primary and companion 128-word histories and returns both next histories. The supplied recipes start each owner at zero and reuse it across two calls. The API takes a long-block-sized input and does not accept or infer `short_flag`; the supported vector profile fixes it to zero. Product history construction/reset ownership and clocks remain open.

Three recipes and six calls compare 3,072 folded values, 3,072 pre values, 192 low-FFT/recurrence values, 84 branch coefficients, 1,536 history values, and 42 raw exponents. The seventh raw-output slot remains a caller canary. The separately bound LFE coefficient-exponent package supplies the zero-energy, clamp, exponent-bit, cap, and no-bias behavior.

The package's formulas are marked `PROTOTYPE_POSTCOMPARE`/static where applicable; native helper output boundaries and whole wrapper outputs were observed, but no internal raw PCM or per-operation buffers were delivered. This does not implement the 256-word input producer, short-block route, owner cold/reset/clock contract, external job selection, ADM path, or whole F09.

Package manifest SHA-256: `96b7b6b5b4d76f4967daa09d3817fcac789a22fba362c505fe915d1aee29f5ca`.

Independent-check SHA-256: `60bcbc859b2b8e0cd487261915c20b38ad3ab3a63dc89851d8a5570da8e07167`.
