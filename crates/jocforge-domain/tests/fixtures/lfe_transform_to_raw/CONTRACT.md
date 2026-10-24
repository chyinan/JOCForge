# Long-block LFE transform to seven raw exponents

## Scope and state

This package covers the selected long-block LFE path from one synthetic 256-word F32 input through primary/companion fold and pre-rotation, a dedicated low-frequency FFT8 gather, recurrence, seven-value post-rotation, and the seven raw LFE exponents. It is not the full-band 128-complex FFT and must not be implemented as “run full-band transform then take seven bins.”

Each controlled call owns separate 128-word primary and companion histories initialized to zero in the recipe and retained across two 256-word calls. `short_flag=0`. The caller-owned history/reset clock for the whole product, actual LFE coefficient-input producer, and short-block route remain open.

## Fold and pre-rotation

For `i=0..63`, process in order:

```text
(a,b,o,s,h) = (2i,   255-2i, 128+2i, 129+2i, 64+i)
(a,b,o,s,h) = (2i+1, 254-2i, 127-2i, 126-2i, 63-i)
```

With `R(x)` denoting one F32 rounding after the operation, compute `A=R(input[a]*window[b])`, `B=R(input[b]*window[a])`, `C=R(input[b]*window[b])`, and `D=R(window[a]*input[a])`; read `old_history[h]` before updating it. Primary writes `temp[o]=R(A+B)`, `temp[s]=old_history[h]`, `history[h]=R(C-D)`. Companion writes `temp[o]=old_history[h]`, `temp[s]=R(A-B)`, `history[h]=R(C+D)`.

For `i=0..127`, `x=temp[2i]`, `y=temp[2i+1]`, and the exact tables `c=long_cosine[i]`, `s=long_sine[i]`:

```text
pre[2i]   = R(-R(c*x) + R(s*y))
pre[2i+1] = R( R(c*y) + R(s*x))
```

## Low-frequency gather and FFT8

For `q=0..15`, build a 16-word interleaved buffer from the eight complex pairs
`(pre[2*q+32*j], pre[2*q+32*j+1])`, `j=0..7`. Apply the following FFT8 independently to each gathered buffer. Split its 16 words into `v0..v3`, each four words, and preserve old operands for each listed vector operation.

1. For pairs `(v0,v2)` and `(v1,v3)`, set `va=R(vb+old_va)` and `vb=R(old_va-vb)` componentwise.
2. Rotate `v3` to `(-v3[1],v3[0],-v3[3],v3[2])`. Then set `old=v0`, `v0=R(v1+old)`, `v1=R(old-v1)`, and rotate the updated `v1[2:4]` to `(-v1[3],v1[2])`.
3. With the current vectors, set `v2=R(v2-v3)` and `v3=R(v3+v2)` using old `v2/v3` values.
4. Let `a=R(root_half*v2[2])`, `b=R(root_half*v2[3])`; set `v2[2:4]=(R(a+b),R(b-a))`. Repeat for `v3`, then set `v3[2:4]=(R(b-a),R(-a-b))`.
5. For each four-word vector `(ar,ai,br,bi)`, replace it with `(R(R(ar+br)*0.125), R(R(ai+bi)*0.125), R(R(ar-br)*0.125), R(R(ai-bi)*0.125))`. Swap the first and second complex pairs of `v1`, then flatten `v0..v3` to 16 words.

`window`, long rotation tables, eight-entry recurrence tables, gather permutation, and `root_half` are the exact bit arrays in `PARAMETERS.json`.

## Eight-bin recurrence and post-rotation

For each `k=0..7`, let `j=gather_permutation[k]`, `previous=(0,0)`, and `current=gathered[15][2*j:2*j+2]`. For `q=14..0`, compute each component as `value=R(R(R(multiplier[k]*current)-previous)+gathered[q][2*j+part])`; then simultaneously set `previous=old_current`, `current=value`.

Compute:

```text
a = R(R(current.real - R(previous.real*recurrence_cosine[k]))
      + R(previous.imag*recurrence_sine[k]))
b = R(R(current.imag - R(previous.real*recurrence_sine[k]))
      - R(previous.imag*recurrence_cosine[k]))
low[2k]   = R(a*0.0625)
low[2k+1] = R(b*0.0625)
```

For the first four bins, primary output is `R(R(long_cos[i]*low[2i+1])-R(long_sin[i]*low[2i]))`; companion reverses the two subtracted products. Both branches share odd outputs:

```text
(target,j,rotation) = (5,5,125), (3,6,126), (1,7,127)
out[target] = R(R(long_cos[rotation]*low[2j]) + R(long_sin[rotation]*low[2j+1]))
```

## Raw LFE mapping and evidence

The two seven-value coefficient rows go directly to the coefficient-pair exponent route, with count 7 and cap 48. For each pair compute separate F32 squares and add, clamp energy to `[-1,1]`, return 24 for zero energy; otherwise use the doubled U32 word exponent rule, clamp the difference to `0..60`, cap at 48, and arithmetic-shift right by one. This route has no bias input. Wrapper return is 0 and output tail canaries remain unchanged in supplied vectors.

The input-only suite has three recipes and two calls per recipe. Wash independently replayed 3,072 folded values, 3,072 pre values, 192 low-FFT values, 84 seven-bin coefficients, 1,536 history values, and 42 raw exponents; every value matched `EXPECTED.json`. Native helper stage boundaries and the complete wrapper output were captured. The inner FFT8/fold/pre equations carry `PROTOTYPE_POSTCOMPARE` plus static evidence; no raw PCM/ADM was used.

Only the supplied finite long-block profile with `short_flag=0` is covered. Invalid pointers/aliases, non-finite/underflow profiles, parent history construction/reset, mode/configuration production, short transform, and whole ADM-to-JOC integration remain open.
