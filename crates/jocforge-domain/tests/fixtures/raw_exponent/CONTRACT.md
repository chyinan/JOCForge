# Raw exponent from POWER bits

## Scope

This contract covers one stateless scalar mapping from a caller-supplied U32 POWER word to one raw exponent value. The input is consumed as integer bits; the helper does not compute POWER from PCM or coefficients. It does not select the route, produce the bias, choose the cap, or establish when the caller should invoke it. Those F09 producer steps remain open.

## Interface and preconditions

Wash interface choice:

```text
raw_exponent_from_power_bits(power_word: u32, bias: i16, cap: i16) -> i16
```

`power_word` carries a binary32 bit pattern but is not converted to floating point. The bounded runtime evidence covers the literal words in `INPUTS.json`, the signed I16 biases `-2, 0, 1, 2, 48`, and cap `48`. The caller must provide a POWER word produced for the POWER route and the correct bias/cap. The source's full pointer/count admission behavior and arbitrary non-finite/negative bit patterns are not covered here.

The scalar interface is a Wash design choice factored from the observed array kernel; it is not a claim about the native function signature. The native array kernel accepts a signed I16 count, processes groups of four followed by a scalar tail, and writes output elements in order. The one-value formula has no persistent state or explicit error/status result.

## Exact integer formula

```text
w = (2 * power_word) modulo 2^32
source_exp = -127                    if w == 0
             (w >> 24) - 126         otherwise
d = clamp(-source_exp, 0, 60)
b = signed_I16_wrap(bias + d)
if b > cap using signed I16 comparison:
    b = cap
output = arithmetic_shift_right(b, 1)
```

Bias addition wraps modulo 65536 and is then interpreted as signed I16. The cap comparison is signed. The final shift preserves the sign. For zero and signed-zero words, the bit kernel still adds bias before applying cap; this differs from the separate coefficient-pair route's zero shortcut.

The selected wrapper supplies `cap = 48`, as reflected in every vector. The constants, exact U32 inputs, signed biases, and expected outputs are hash-bound in `PARAMETERS.json`, `INPUTS.json`, and `EXPECTED.json`.

## Validation boundary

The static bit rule is `STATIC_CONFIRMED`. Native runtime evidence covers 5 recipes and 50 output values; Wash independently recomputed all 50 from the integer formula and matched `EXPECTED.json`. This does not validate the actual audio-to-POWER writer, mode/format route selection, production bias provider, arbitrary count/pointer errors, coefficient-pair path, LFE behavior, or the whole F09 pipeline. Independent product implementation and audio acceptance remain `NOT_RUN`.
