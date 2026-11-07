# LFE 16-point F32 follow-on transform

## Result

`FUN_180229b00` transforms the selector-zero work vector into the terminal F32[16] input for the existing seven-F32 main/companion projection. It gathers 16 rows, applies `FUN_18022a0b0` to each row, runs an 8-bin three-pass scalar recurrence, then writes 16 F32 values to `param_1[0..15]`. Main reaches it through `FUN_180229d90` before `FUN_18022a310`; companion reaches it through `FUN_180229de0` before `FUN_18022a410`.

## Gather

Let `X[0..255]` be the F32 input to `FUN_180229b00`. Its first loop gathers one 16-F32 vector per row `r=0..15`:

```text
V_r[2m]   = X[32m + 2r]
V_r[2m+1] = X[32m + 2r + 1]   for m=0..7
```

This maps every X index 0..255 exactly once. Each V_r is transformed in place by `FUN_18022a0b0`, and the 16 outputs are stored row-major as `T[r][0..15]`.

## `FUN_18022a0b0` butterfly equations

The function uses `F32` arithmetic. `flip(v)` below is the code’s U32 sign-bit XOR with `0x80000000`, not an arithmetic negate. The stages follow the recovered assignment order.

Stage 1, from V to A:

```text
A[0..3]   = V[0..3] + V[8..11]
A[4..7]   = V[4..7] + V[12..15]
A[8..11]  = V[0..3] - V[8..11]
A[12]     = flip(V[5] - V[13])
A[13]     = V[4] - V[12]
A[14]     = flip(V[7] - V[15])
A[15]     = V[6] - V[14]
```

Stage 2, from A to B:

```text
B[0] = A[0]+A[4]       B[4] = A[0]-A[4]
B[1] = A[1]+A[5]       B[5] = A[1]-A[5]
B[2] = A[2]+A[6]       B[3] = A[3]+A[7]
B[6] = flip(A[3]-A[7]) B[7] = A[2]-A[6]
```

Stage 3, from B to C:

```text
C[8..11]  = B[8..11] - B[12..15]
C[12..15] = B[12..15] + B[8..11]
```

For the in-place rotation stage, let `q=0x1.6a09e6p-1`; the two products in each sum are separate F32 multiplies:

```text
q10 = q*C[10]       q11 = q*C[11]
q14 = q*C[14]       q15 = q*C[15]
C[10] = q10 + q11
C[11] = q11 - q10
C[14] = q15 - q14
C[15] = flip(q14) - q15
```

The final stage applies `s=0x1.0p-3` to pairs `(0,2)`, `(1,3)`, `(4,6)`, `(5,7)`, `(8,10)`, `(9,11)`, `(12,14)`, and `(13,15)`. For each pair `(i,j)`, it writes `(C[i]+C[j])*s` to index i, then `(C[i]-C[j])*s` to index j.

## Eight-bin recurrence and terminal F32[16]

For output bin `k=0..7`, let `p=P[k]` from the U32 index table and `j=2p`. Set `c=F[k]`, `r=0`, `q=0`, `a=T[15][j]`, and `b=T[15][j+1]`. The decompiler stack offsets map `pfVar6` to `T[13][j+1]`; its `-0x50` F32 step selects row groups with top rows 14, 9, and 4. For each `top` in `(14,9,4)`, run the following 10 F32 updates in order:

```text
r1 = (c*a - r) + T[top][j]
q1 = (c*b - q) + T[top][j+1]
a1 = (c*r1 - a) + T[top-1][j]
u1 = (c*q1 - b) + T[top-1][j+1]
b1 = (c*a1 - r1) + T[top-2][j]
v1 = (c*u1 - q1) + T[top-2][j+1]
r  = (c*b1 - a1) + T[top-3][j]
q  = (c*v1 - u1) + T[top-3][j+1]
a  = (c*r - b1) + T[top-4][j]
b  = (c*q - v1) + T[top-4][j+1]
```

Here `r,q,a,b` carry between the three row groups for this bin; they are reinitialized for the next k. The function returns the pair:

```text
out[2k]   = ((a - r*W0[k]) + q*W1[k]) * 0x1.0p-4
out[2k+1] = ((b - r*W1[k]) - q*W0[k]) * 0x1.0p-4
```

`P` maps output bins to column pairs `j=[0,8,4,12,2,10,6,14]`. The function writes only `param_1[0..15]` (16 F32 values, 64 bytes) at 4-byte stride after gathering all 256 input F32 values.

## Exact constants and hashes

All F32 values below are exact hexadecimal values and raw IEEE-754 bit patterns from Ghidra’s read-only static ranges. The full ranges are hash-bound in `EVIDENCE.json`.

| Table | RVA | SHA-256 | Entries (hex F32 or U32) |
|---|---:|---|---|
| `W0` | `0x6be1f0` | `7ce70d170aa062097ee0a0f0e3f8426492e92869586d42df885d86f3c504da73` | `0: 0x1.0p0/3f800000; 1: 0x1.ff621ep-1/3f7fb10f; 2: 0x1.fd88dap-1/3f7ec46d; 3: 0x1.fa7558p-1/3f7d3aac; 4: 0x1.f6297cp-1/3f7b14be; 5: 0x1.fa7558p-1/3f7d3aac; 6: 0x1.fd88dap-1/3f7ec46d; 7: 0x1.ff621ep-1/3f7fb10f` |
| `W1` | `0x6be210` | `a05e11182771bd5f84f24fd8fa19ea0c08f6f8b7f9c8e75c4f262b78b2f7321c` | `0: 0x0.0p0/00000000; 1: 0x1.91f66p-5/3d48fb30; 2: 0x1.917a6cp-4/3dc8bd36; 3: 0x1.2c8106p-3/3e164083; 4: -0x1.8f8b84p-3/be47c5c2; 5: -0x1.2c8106p-3/be164083; 6: -0x1.917a6cp-4/bdc8bd36; 7: -0x1.91f66p-5/bd48fb30` |
| `F` | `0x6be230` | `2f13fb2ce5ce3b90a27b09b8e1ea543fe2bb5625f699d124cf37edfa8716d007` | `0: 0x1.0p1/40000000; 1: 0x1.ff621ep0/3fffb10f; 2: 0x1.fd88dap0/3ffec46d; 3: 0x1.fa7558p0/3ffd3aac; 4: 0x1.f6297cp0/3ffb14be; 5: 0x1.fa7558p0/3ffd3aac; 6: 0x1.fd88dap0/3ffec46d; 7: 0x1.ff621ep0/3fffb10f` |
| `P` | `0x6be250` | `4866c8cbca474f15e4f34dc44f97411ade60497227c9a68625f1dcec8dabc3d3` | `[0,4,2,6,1,5,3,7]` (U32) |
| `s_out` | `0x8a90cc` | `b1801134f2c71f5540537dc8ab78eb44398b15f76af4abf4df3a24468d8e50d6` | `0x1.0p-4/3d800000` |
| `s_bfly` | `0x8a9104` | `31b67dba7cfd6e2d7540f9c96d90a45b8f2d44956620723024d4e1beeacd4602` | `0x1.0p-3/3e000000` |
| `sign_mask` | `0x8a9670` | `6d58692645c9d1cfaf13541cbd258f86193ef63c2f1d38f6bbca9617372d7bd6` | `0x80000000` (U32 bit mask) |
| `q` | `0x9d1f9c` | `723c8fdc213c6caa05324e5d35fed7b7ba12565f07d26abf773735a424004953` | `0x1.6a09e6p-1/3f3504f3` |

## Machine-operation and rounding scope

Ghidra instruction listings show 0 FMA mnemonics in both functions. `FUN_180229b00` has 40 scalar single-precision arithmetic instructions (`MULSS/ADDSS/SUBSS`) and `FUN_18022a0b0` has 40, plus 4 `ADDPS` and 4 `SUBPS` packed-F32 operations in the butterfly. Neither function contains `LDMXCSR` or `STMXCSR`. Therefore multiplication and addition/subtraction are separate F32 instructions in this build; fused multiply-add is not observed. The active MXCSR rounding/FTZ/DAZ state is runtime state and was not captured, so a specific rounding mode is not claimed. Sign changes use the U32 mask rather than an arithmetic negation.

## Connection and scope

The wrappers call `FUN_180229b00` on their local work buffer after their guard sequence; its 16-F32 output is the terminal input to `FUN_18022a310` (main) or `FUN_18022a410` (companion). The main/companion input vectors may differ upstream; this report does not infer equality or parity between their live buffers. The separate 6×6 matrix branch and the wire writer were not analyzed.

## Input-only vectors

`INPUT-ONLY-VECTORS.json` contains a zero vector, all 16 one-hot basis vectors, and an integer ramp. Outputs are omitted, and no vector was executed.

## Evidence disposition

Static Ghidra 12.2 read-only only. No DEE process was launched; no PCM, EC3, license, or key content was read. The sanitized EVIDENCE.json records source hash provenance and the static-only NOT_RUN status.
