# Prefix contribution precision reference

Ready scope: F04A.4 prefix-contribution arithmetic and synthetic previous/current half recomputation. This adds an independent mathematical validation reference for existing approved rules, not a new audio algorithm.

Compute each decimal weight as a binary64 literal, take nearest-even binary64 sqrt, then round the root to F32 before multiplying gain. The two roots are F32 words 0x3f6c0535 and 0x3ec64bf8. Each subsequent multiplication/addition has its own F32 rounding point, preserving the order in NUMERIC-RULES.json; no FMA or reassociation is allowed.

For these two particular numbers, first rounding the literal to F32 and then promoting it for sqrt produces the same final F32 roots. That fact does not justify arbitrary early rounding, but it also cannot by itself explain differing output. Retaining the sqrt result in F64 through gain multiplication is a different expression and the supplied synthetic suite detects differences. Do not conflate literal-input precision with the mandatory root-to-F32 conversion before gain.

The clean reference uses exact Fraction arithmetic, integer quotient/remainder nearest-even rounding and exact squared-midpoint tests for binary64 sqrt. It does not load frozen prototype code, use platform sqrt, read implementation output, or contain expected audio arrays. `prefix_numeric_reference.py` is approved as a validation-only mathematical oracle/input generator. Production code must continue to implement the approved F32 operations itself; it must not depend on or call this Python tool.

Previous and current prefix halves are independently cleared and recomputed with this callback's gain, mode and branch. Do not reuse an old previous-prefix sum or reconstruct it by subtraction. Filter activation, continued state and reactivation remain the approved F04A/F04F rules; the scalar oracle does not replace those filters or their providers.

The primitive defines no EOF schedule. Its parent uses the applicable existing main F02 cached-drain or independent meter F06 zero-input-drain contract. Neither schedule is inferred from these numerical cases.

The implementation Agent reports that the current isolated expression already has the correct pre-gain R32 and matches these two root words. No mathematical change is indicated by that report alone. Earlier expressions and a separate correction commit are not available in the current reported history; record that historical limitation rather than asserting a reconstructed cause. Do not restore an old actual trace as an expected oracle. The whole corrected g0-g2 DSP chain remains outside this local reference's conformance claim.
