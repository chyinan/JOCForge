# F08NZ nonzero transform composition

## Scope

This clean local contract composes the delivered `math-generated-v1` F08 provider with nonzero synthetic block recipes. It verifies the path

```text
nonzero block + old 128-word bank state
→ F08 fold/window
→ long or two-short pre-rotation
→ staged normalized FFT
→ long or interleaved-short post-rotation
→ primary/companion spectra
→ F09 POWER and PRIMARY raw exponent inputs
```

LFE uses the same bank-specific fold and long pre-rotation, followed by FFT8, recurrence, seven-bin post-rotation and the same POWER/PRIMARY exponent input rules.

The authoritative transform operations and rounding points remain N00 and F08. This document fixes only the input recipes, adapter validation, trace identities and acceptance assertions used by this delegated review.

## Parameter binding

The only transform provider is `parameters/ordinary-transform-math-v1.json`, exactly 16,538 bytes with SHA-256 `f1fb1a16b3caedab6ddc583201b9454755fd8871ecdf7881a6a3ffbad10fddf1`.

The adapter validates schema/profile, exact file identity, every required field, fixed dimensions, finite F32 words, the 0..127 permutation bijection, its even-index 0..63 bijection, and the 0..7 LFE permutation. Missing fields return `PARAMETER_UNAVAILABLE`; identity, length or permutation errors return `PARAMETER_IDENTITY_ERROR`.

## Synthetic input recipe

For each recipe and sample index `i`, the exact input is

```text
F32(((i × multiplier + offset) mod modulus − centre) / power_of_two_denominator)
```

The denominator is a power of two, so the recipe is deterministic without decimal ambiguity. It is generated data, not WAV/PCM or a song-derived lookup.

## State and branch connection

- Primary and companion each own a separate 128-word F32 history.
- A call reads the whole old history and returns a new history; it never mutates caller state on failure.
- FBW short flags use external strict I1 values 0/1. Language booleans are rejected. `0` selects the long path; `1` selects the two-short path for both banks.
- The continuous acceptance sequence is long → short → long. Switching branch never resets either bank history.
- LFE always uses the LFE route and has no FBW short flag. Its two banks remain separate.
- Reset recreates all four 128-word states as positive zero. Replaying the first FBW/LFE block after reset must reproduce the cold spectra, exponent inputs and next states exactly.

## Exponent inputs

For each selected output bin, POWER is `min(1,R32(R32(primary²)+R32(companion²)))`. Its raw exponent follows F09 POWER rules. PRIMARY consumes only the primary spectrum, applies the F09 full-band clamp for FBW, and produces its independent raw exponent. LFE reports the seven raw inputs without extending the unclosed native out-of-range mantissa rule.

Every successful case records U32 hashes for input, fold, full spectrum, exponent-consumed prefix, primary raw exponent row, POWER F32 row, POWER raw exponent row and both next histories. A same-input discriminator runs long and short from the same nonzero input/history and requires differences in spectrum words and at least one exponent-input domain.

## Errors and commit boundary

Input/history dimensions, finite input, and strict I1 flags are validated before transform work. These failures return `F08_INPUT_ERROR`. A non-finite arithmetic result returns `F08_NUMERIC_FAILURE`. Failure returns no spectrum or next state and leaves the caller-owned state byte-identical.

## Acceptance boundary

This review can close deterministic composition for the delivered math-generated provider. It cannot establish historical D01/D04 identity, native equivalence, encoded audio/render quality, whole-codec acceptance, or whole-package readiness. The generated provider itself records 12 KBD/window word differences and differing LFE trigonometric values relative to the research identity; those remain explicit production-equivalence gaps.

The fresh r1 package-only result is registered in `test-vectors/F08-NONZERO-TRANSFORM-LOCAL-REVIEW-STATUS.json`. It independently matches every exposed trace hash/preview, reset assertion and machine error without reading the verified Python model. The frozen 4,650-word postcompare remains a reported model comparison whose public identities are consistent; it is not reclassified as an independent word-by-word replay because those expected words are absent from the freeze.
