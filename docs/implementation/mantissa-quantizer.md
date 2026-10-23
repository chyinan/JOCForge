# F12S stateless mantissa quantizer

This direct slice implements only `finite_f32_word + bap -> code/error` for the selected F12S mantissa quantizer. The input word is already the scaled binary32 coefficient supplied to the quantizer. Exponent scaling, transform and fit production, BAP allocation, trial search, grouping, bitstream construction, frame assembly, state, and clocks remain outside this module.

The implementation follows the hash-bound F12S.5 rules in the copied package spec and reuses the domain's explicit R32 add/multiply and truncation primitives. BAP 1/2/4 use the specified clamped truncation; BAP 3/5 reject their reserved sentinel codes; BAP 6..15 use the specified width-dependent fixed-point offset, separately rounded binary32 operations, ties-to-even integer rounding, arithmetic shift, and signed-16 clamp. BAP 0 returns code zero after input validation.

The local test fixture preserves the complete sanitized package payloads and verifies the source manifest, independent-check binding, and every payload byte count and SHA-256 before executing the 208 quantizer cases. The vector set has 204 successful codes and four reserved-sentinel failures. Additional local tests exercise non-finite input, unsupported BAP, and a non-finite intermediate.

This is an implementer-code result for one pure function, not native/runtime replay. It does not mark F12S, F12, or the product ready. The package's whole-readiness value remains `NO`.
