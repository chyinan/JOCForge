# LFE coefficient-pair exponent route

This direct slice implements the washed selected 5.1 LFE coefficient-pair route for exactly two blocks of seven finite coefficient pairs. It squares real and imaginary F32 values separately, adds them in F32, clamps the resulting energy to `[-1.0, 1.0]`, then computes the LFE exponent directly from those energy bits. Exact zero energy takes the explicit `24` output path. For nonzero energy, the route doubles the F32 bit word modulo 2^32, extracts the source exponent, clamps the negated exponent to `[0, 60]`, applies the signed I16 cap of `48`, and arithmetic-shifts right by one.

The function has no mode, format, or bias input: those are caller-side full-band choices and the supplied recipes show that LFE output is invariant across the tested mode/format changes and bias values `0`, `2`, and `4`. This route is implemented independently from the full-band POWER-row helper and does not consume the separate POWER probe values in the fixture. The latter are explicitly marked as separate native-helper outputs, not LFE internals.

The two output blocks each expose seven written I16 slots and one caller-owned tail canary slot. Calls overwrite all seven bins in both blocks, including consecutive reuse. In each recipe/call, block 0 output also matches the independently recorded block-0 LFE output from a combined full-band/LFE invocation.

Five input recipes, two consecutive calls, and two LFE blocks cover 140 LFE outputs; the fixture also binds 140 separate POWER probe values and 70 combined-call LFE outputs. Package manifest SHA-256: `63d6fa945445ea9545b3ac02ad9df86b832a4abcccf6beb5caaf515ed1a3540a`. Independent-check SHA-256: `9321c12d5414febcce241e9102628434dc7ab6b1c0e1c4a1168b2037ba6fc258`.

Coefficient production, job-to-LFE selection, other profiles, invalid pointers/lengths, non-finite or wider input domains, parent lifecycle, whole F09, product integration, and audio acceptance remain outside this slice or `NOT_RUN`.
