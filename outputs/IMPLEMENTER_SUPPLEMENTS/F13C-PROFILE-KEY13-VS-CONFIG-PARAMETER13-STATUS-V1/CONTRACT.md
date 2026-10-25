# Wrapper profile key 13 and configuration parameter 13

Status: `FIELD_NAMESPACES_DISTINCT_BY_EVIDENCE; CROSS_FIELD_JOIN_UNBOUND`.

Two hash-bound evidence packages use the integer label 13 for different fields:

- The encoder wrapper's signed-I32 profile-pair list contains key 13 with value expression `X + Y`. The meanings of `X` and `Y`, and the selected wrapper invocation, are unbound.
- A configuration record has parameter id 13 of type signed I32. Its reviewed count producer computes `non_lfe_count + lfe_presence`; the selected capture recipe produced 16.

No direct producer/consumer edge, shared owner, call token, or source-generation identity connects these fields in the reviewed packages. Both packages bind to the same encoder image; common module identity does not make their fields the same. Numeric label 13 or a possible equal value does not prove a mapping.

Keep the two inputs distinct. The cross-field relationship, selected wrapper invocation, and shared generation remain `UNBOUND`. This status does not change an algorithm or establish a global absence claim.
