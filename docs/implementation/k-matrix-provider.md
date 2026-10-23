# F06KG selected K-matrix provider

This direct slice generates the selected 48 kHz, modern, two-stage K-filter provider from the delivered exact-decimal coefficients. It expands the eight ordered basis vectors through four recurrence lanes using exact rational arithmetic, applies the proved shelf gain only to stage-0 numerator coefficients, rounds each matrix entry to decimal 1e-8 ties-to-even, then converts that rational exactly to binary32 ties-to-even. The provider returns 64 matrix words, eight positive-zero cold words, and the selected lower/upper/noise state bounds.

The production input JSON is a hash-pinned input-only extraction: expected `generated_f32_words` are deliberately absent. Tests compare the extracted inputs to the approved source parameter asset, then compare all generated 72 words and their complete 288-byte F32LE identity against the separately pinned source expectations.

This is provider/generator implementation only. It does not implement F06W.3/.4 sample consumption, windowing, state evolution, power/meter calculations, ADM/frontend composition, source/AU clocks, or parent transaction. The package's continuous 6-block consumer assertions remain outside this slice.

The source package marks these coefficients and bounds `INTERNAL_ONLY` and explicitly says public or third-party distribution is not authorized. This implementation is local and uncommitted; it must not be committed to or pushed on the public branch without separate authorization.
