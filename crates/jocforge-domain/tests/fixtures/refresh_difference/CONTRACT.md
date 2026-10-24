# F09E D06-E refresh-difference map generator

## Slice boundary

This package defines only the stateless integer function that maps one refresh-difference index to an I16 table value. It does not implement exponent refresh, coefficient planes, BAP allocation, F10 normalization, frame planning, or policy-provider loading.

## Input and output

- Input: `index`, a strict integer in the closed range `0..=31`.
- Boolean values are not integers for this interface. Floats, null, strings, and other types are rejected.
- Output: signed I16 value, computed with exact integer arithmetic.
- No floating-point operations, state, clock, provider lookup, or serialization side effect occurs in the mapping function.

## Formula

```text
if index < 2: 0
else if index == 2: 1
else: 2 * index - 4
```

The generated table serializes the 32 outputs in ascending index order as signed I16 little-endian words. The serialized table is 64 bytes; its SHA-256 is recorded in `PARAMETERS.json`.

## Acceptance cases

The source vector file has 32 valid cases, corresponding to indices `0..=31`, and 7 rejected inputs: `-1`, `32`, `true`, `false`, `1.0`, `null`, and the string `"2"`. Every rejection is `EXPONENT_INPUT_ERROR`.

For stable test names, an implementation may label valid cases `D06E_VALID_00` through `D06E_VALID_31` by input index, and rejections `D06E_REJECT_NEGATIVE`, `D06E_REJECT_UPPER_BOUND`, `D06E_REJECT_TRUE`, `D06E_REJECT_FALSE`, `D06E_REJECT_FLOAT`, `D06E_REJECT_NULL`, and `D06E_REJECT_STRING`.

## State, errors, and exclusions

The function is pure and stateless. Invalid inputs return `EXPONENT_INPUT_ERROR` with no output value. No reset, commit, clock, or owner protocol applies.

The 32-entry map is a research-observed mathematical behavior rule, not an independent original design claim. It has 32/32 entry agreement in the delivered bounded comparison. This does not claim whole F09/F10 equivalence or producer readiness.

Delivery scope is `INTERNAL_ONLY`; no public or third-party distribution authorization is asserted by this package. Whole-product `IMPLEMENTATION_READY=NO`.

