# Final F06 selection kind to ID1 preferred

Ready scope: completed F06 scalar selection -> public measurement kind -> programme ID1 preferred. This closes that typed field connection for the approved scalar domain. It does not certify upstream native metering DSP or a historical selected native run.

## Finalized typed inputs and selection

Use the same final, programme-owned measurement inputs as approved F06.2/F06U. Normalize each host scalar by R32(host/128). The finite normalized F32 tuple is threshold T, speech percentage P, dialogue candidate A, ITU candidate B, and freshly computed ungated candidate C. The selector chooses and records its origin in the same branch:

| Condition | Selected normalized value | Origin | Public measurement kind | ID1 preferred |
|---|---|---|---|---|
| (P <= T or A <= -0.46875) and B <= -0.9375 | C | Ungated | 0 | 0 |
| (P <= T or A <= -0.46875) and B > -0.9375 | B | ITU/level | 2 | 0 |
| P > T and A > -0.46875 | A | Dialogue | 1 | 1 |

Threshold equality takes the fallback branch. Candidate selection preserves the chosen normalized F32 bits; host publication is a separate R32 multiplication by exact 128. The finite-domain contract includes signed zero and finite subnormal inputs. Nonfinite inputs, invalid lifecycle state and publication overflow are not admitted by this equivalence statement; apply the existing clean measurement admission rules.

## Native provider correspondence

Both recovered provider cores write selected scalar and selector-origin together using the conditions above. Selector-origin codes Ungated/ITU/Dialogue are 0/1/2. Their public conversion S maps 0->0, 1->2, 2->1, and preserves the intermediate unmeasured sentinel -1. The prepare path returns this public kind directly.

The processor path additionally maps public values through wrapper W: -1->0, 0->1, 1->2, 2->3. Its root getter G maps 1->0, 2->1, 3->2, otherwise -1. Thus G(W(S(origin))) equals S(origin) on the reachable domain. Both paths publish the same chosen host metric and public kind when supplied the same completed finite tuple and valid finalized lifecycle. Their ID1 conversion is strict I32(public_kind==1).

This correspondence is established by paired scalar/origin writes and the actual conversion/getter chains, not by names or equal numeric codes. It justifies the branch-independent typed conversion on this domain. Observing which historical factory branch ran is unnecessary for this field connection. It does not prove that different provider histories produce equal input tuples, equal errors, equal upstream DSP, or equivalent native restart/flush behavior.

## Minimal internal interface decision

Clarify the existing final programme measurement `gating_kind` as the public measurement result domain: Ungated=0, ITU=2, Dialogue=1. If its current Rust type is ambiguous, use a semantic type or a checked explicit conversion. Produce it beside selected loudness from the same F06 selector branch, carry it in the existing final measurement object, and derive preferred by comparison with the Dialogue variant. Do not add a user/XML/UI field, additional DSP state, or a new stored origin solely for this connection. The internal selector-origin enum 0/1/2 cannot be directly cast to the public enum.

The final result is bound after the complete independent programme measurement. Construct the ID1 configuration once and reuse it for source timer updates. EOF does not replace measurement or preferred; cold model restart reconstructs the timer/queues according to the approved lifetime contract. Current F06U ungated fallback remains mandatory when ITU is unavailable.

An intermediate unmeasured -1 and the no-winner aggregate's locally initialized zero are not evidence of a measured Ungated selector branch. They must not fabricate a completed programme measurement. Prepare's finalized-getter guard and the processor lifecycle remain their own admitted-state conditions; identical failure statuses are not claimed.

## Evidence and remaining programme work

The evidence grade is cached static analysis plus exact frozen model source identities and synthetic value checks. Wash verified six native-package payloads, three cached exports, ten recovered code-body hashes, three functional constants, four approved contract identities and the base chain manifest. Normal and optimized checkers each passed 12 input-only cases and 10,012 synthetic equivalence checks, covering all three origins and five typed-input rejections. Original PE rehash and native execution remain NOT_RUN.

Source field semantics and this conversion are now specified. Historical selected factory values, upstream scalar production from real ADM, and wider native source/reset identity remain separate evidence questions; they do not hold this completed-scalar conversion open.
