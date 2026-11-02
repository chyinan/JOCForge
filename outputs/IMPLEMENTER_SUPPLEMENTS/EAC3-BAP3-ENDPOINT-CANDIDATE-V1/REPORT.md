# E-AC-3 BAP-3 positive-endpoint candidate

## Washed behavior

Ghidra 12.2 static analysis of the hash-bound DEE encoder image supports a BAP-3 quantizer path with scale 3.5, seven levels, and a three-bit output. The computed integer is clamped to [0,6] before packing. At coefficient +1.0 with exponent 0, BAP 3 produces raw 7, clamps to 6, and packs 110.

The current Wash-approved F12S behavior treats this raw 7 as a reserved sentinel and rejects the candidate. That is a concrete model-rule split.

## Scope

The single-bin vector in INPUT-ONLY-VECTOR.json distinguishes the two rules. It was not executed in native DEE or Rust, and this tuple's occurrence in the matched 125-AU sample window is unobserved. This is therefore a high-value audio-core A/B candidate, not an attribution of the current channel-3 residual.

The smallest next experiment is to add the single vector at the existing quantizer seam, then run the current Human Cat input through the matched full-file controls and compare the same AU window. Do not alter the separate window table or DRC controls in that A/B.
