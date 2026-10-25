# Native profile wrapper static input contract

Status: `WRAPPER_INPUT_SYNTHESIS_STATIC_CLOSED; CONVERTER_HANDOFF_AND_SELECTED_INVOCATION_UNBOUND`.

## Wrapper-local profile input

On the reviewed encoder-wrapper path, the wrapper synthesizes a profile input with one profile and five signed I32 key/value pairs, each pair occupying 8 bytes. The first pair is explicitly `(1, 10)`, so this path selects native profile type 10; it does not obtain type 10 from the default-when-key-1-is-absent rule. The fixed prefix bytes decode as signed I32 words `[1, 10, 11, 7]`.

The remaining pairs are assembled as follows:

- key 13 receives the sum of two opaque wrapper I32 inputs `X + Y`;
- key 14 receives opaque input `Y`;
- key 12 receives the wrapper's conversion of an encoder configuration field into a rate word.

The meanings and selected values of `X` and `Y`, the rate-word conversion rule, and overflow behavior for `X + Y` are not established here. Do not derive them from ADM track, object, or channel counts.

The normalizer reads this same pair list, stores type 10 in a normalized profile, and the row builder installs one row (`row0`) in a profile root. `row0` links to that normalized profile. On this construction path, the type-specific constructor callback is invoked on the same `row0` after the five event-list descriptors are installed. These are static same-construction facts; the selected host job and later parent invocation are not bound to this path.

## Partial flag and list selector

For this type-10 wrapper path, the normalizer directly sets the native partial-block flag to I32 value 1. The list-selector dividend is a local I32 scalar `A`, initialized to zero and passed by address to an internal context constructor. That constructor writes a computed value through the same output parameter and performs internal block rounding before returning. The row builder then computes:

```text
q0 = trunc_toward_zero(A / 1536)
q  = q0 + 1, if partial_flag != 0 and A != 1536*q0
```

For nonnegative `A`, this is floor division followed by one partial-block increment. The builder stores `q` on the profile row and uses it as the event-list selector. Five list descriptors exist, so the descriptor is valid only when `0 <= q < 5`.

The source/dimension inputs to the internal constructor, the units and clock meaning of `A`, its selected numeric value, and whether selected `q` is within `[0,5)` remain unbound. The V4 label `a7` does not prove that `A` is an externally supplied ADM time parameter. The native partial flag is not an ADM jump/interpolation field, authored mode, or Control52 source-mode byte.

## Open host and selected-run identity

The converter-side ADM storage owner and the encoder wrapper are separate static owners in the reviewed package. No typed host/plugin call or common job/source token joins the converter's ADM/CHNA description to this encoder wrapper input. Therefore type 10 is conditional on this wrapper being invoked; it is not yet the actual selected profile type for the measured input.

The selected wrapper invocation, source generation, actual `A`, and selected list selector remain `UNBOUND`/`NOT_CAPTURED`. This contract establishes no product-ready fixture, whole F13C readiness, or algorithm change.
