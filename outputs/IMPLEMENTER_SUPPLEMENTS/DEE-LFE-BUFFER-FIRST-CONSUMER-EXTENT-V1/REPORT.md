# LFE buffer first consumer extent

## Bound selected invocation

The upstream Ghidra receipt reuses the alignment-helper path and traces a later call from FUN_140340640 to FUN_14036bc60. For this invocation, RCX is the original 256-F32 buffer, EDX=1, and R8D=0x100. The selected code path reads four scalar F32 values from RCX offsets 0, 4, 8, and 12, caches them on the stack, and uses that local vector for 64 loop iterations. A count comparison skips the later RCX input loops. The input read extent for this call is therefore 16 bytes, indices 0..3.

The function has no direct call and no memory store through RCX; it returns a scalar in XMM0. This package does not assign mathematical meaning to that scalar.

## Remaining LFE path

The dispatcher has later helper calls that also receive the buffer pointer. Their effects are not analyzed here, so content continuity to e350 remains UNBOUND. No sample-specific branch, numeric parity, or correlation cause is established.

The upstream manifest SHA-256 is c13bad284c9b330e1fb70823a51a91ddc4ee4af874f9432a09dea4d2c723f751 and package-seal SHA-256 is e290d1befc669075da4460af1204fd011c834a1bda832a27f8eaf3c88e04c86e. Its checker passed normally and under python -O. This supplement excludes raw listing material, media, PCM, payload, key, and license.
