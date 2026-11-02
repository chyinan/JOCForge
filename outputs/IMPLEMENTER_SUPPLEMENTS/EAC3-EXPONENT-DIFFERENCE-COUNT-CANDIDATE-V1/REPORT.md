# E-AC-3 selected-stride exponent difference count

The native consumer reads D as a signed 16-bit field at
`state+0x1418c+10*block+2*lane` and performs D exponent-difference iterations.
The selected stride changes which exponent samples are visited; D counts
differences among those selected samples, not raw row positions. A reset path
zeros the six-by-five D grid, but the positive D writer and exact reset/frame
writer instance join were not found.

The best model candidate is `D=max(0,K-1)`, where K is the exponent sample
count after stride selection, including the initial seed. Combined with the
separate U7 group-count candidate `N=(D+2)//3`, the included synthetic row
selects five exponent samples at stride 2, yielding candidate D=4, N=2, and
two tail values. Both producer rules remain inference-only; this vector is
arithmetic-only, not a native observation.

Use the pair as an explicit model candidate for focused lane vectors and
compare across authorized ADM examples. Do not mark native producer parity or
full-frame conformance from this scan.
