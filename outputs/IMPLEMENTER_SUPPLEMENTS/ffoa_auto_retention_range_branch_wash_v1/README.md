# FFOA auto retention and range branch

This Wash-only addendum closes one static branch: the input schema defaults `ffoa` to `auto` and declares that this retains the source FFOA. The generic property loader skips the explicit setter for an `auto` value. The first-frame-of-action file-position consumer uses the source values separately and computes `max(offset, ffoa) - offset` seconds before sample-rate conversion.

R138's authorized 38-channel reference has equal `offset` and `ffoa` values of 3,610 seconds, so it does not distinguish the branch priority. The seven authorized ADM metadata audit found a common programme-start token; it did not expose a distinct `offset`/`ffoa` pair. The source metadata virtual writer and the `offset=auto` resolution remain unbound.

The range-origin rule is a must-have for selecting the exact samples that reach the codec. The first typed spatial/fit consumer remains unobserved. No native run, model replay, Type10/six-plane composition, or Gate 1 closure is claimed.

Verify with:

```text
python verify_ffoa_auto_retention_range_branch.py
python -O verify_ffoa_auto_retention_range_branch.py
```
