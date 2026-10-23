# F12S mantissa quantizer slice

This package closes one pure, stateless slice only: `finite_f32_word + bap -> status/code` for the F12S mantissa quantizer.

Included evidence:

- 208 quantizer cases with exact expected codes;
- accepted BAP values and sentinel/error cases;
- the selected F12S arithmetic rules in the included specification;
- package-only verification and local review identities.

The implementation must not add any of the following to this slice:

- audio transform or fit producer;
- exponent planning or BAP allocation;
- profile search or retry;
- grouping, bitstream/header/frame construction;
- EMDF, CRC, source/AU clock, or parent transaction state;
- interpretation of raw song/reference data.

`production_usable=true` applies only to this stateless quantizer function after local implementation tests pass. Whole F12 and whole-product readiness remain `NO`.

