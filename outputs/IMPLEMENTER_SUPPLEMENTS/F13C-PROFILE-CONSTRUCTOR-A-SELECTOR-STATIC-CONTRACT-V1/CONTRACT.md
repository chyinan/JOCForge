# Conditional profile-constructor A and selector rule

Status: `TYPE10_WRAPPER_A_AND_Q_STATICALLY_CLOSED; SELECTED_INVOCATION_UNBOUND`.

On the reviewed encoder wrapper path, the signed I32 input `p` is read from the normalized profile's argument-15 slot. For the type-10 wrapper pair prefix `(1,10),(11,7)`, the pair normalizer writes `p=0`, selects profile type 10, and sets the type-10 partial-block flag to I32 `1`. The row builder passes the address of a local signed I32 `A` to the internal context constructor as its output parameter. The wrapper's key-11 value supplies mode 7.

For this conditional type-10/mode-7 path, signed IDIV truncates toward zero. The constructor first computes `sum1 = p + 1312`, rounds the quotient up by one only when the remainder is nonzero, and forms `delta1 = 1536*q1 - sum1`. When `delta1 >= 0`, it constructs `pre = p + 2624 + delta1 + 1280`, then unconditionally advances `pre` to the next 1536 multiple using signed truncation toward zero. With the normalized `p=0`, every intermediate fits signed I32 and the result is `A=4608`.

The row builder then computes `base_q = trunc_toward_zero(A/1536)`. If the partial flag is nonzero and `A != 1536*base_q`, it increments once. For the conditional wrapper result `A=4608`, the remainder is zero and `q=3`, which is valid for five descriptors (`0 <= q < 5`).

`A` is an internal constructor output in 1536-unit arithmetic. Its units or clock meaning are not established; do not call it an ADM timestamp or an externally supplied `a7`. The actual host invocation of this type-10 wrapper, selected runtime `A/q`, and source generation remain `UNBOUND`. The static result does not establish that the measured input used this path or authorize a production parameter without the host-to-wrapper join.
