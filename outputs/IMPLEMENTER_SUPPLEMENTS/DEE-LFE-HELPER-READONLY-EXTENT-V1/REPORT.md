# LFE alignment-helper read-only extent

## Bound helper behavior

The two alignment-selected helpers receive the same unadjusted RCX pointer and EDX=0x100. The aligned helper reads sixteen-byte vectors from the start of the buffer, then reads the remaining vectors in its loop. The unaligned helper reads scalar F32 values in groups and handles a three-value tail. Both read exactly 256 F32 values, or 0x400 bytes, from RCX through RCX+0x3ff.

Neither helper contains a direct call or a memory destination. Each computes a scalar min/max reduction and returns it in XMM0. The input buffer is unchanged inside these helpers. The loop structures establish that EDX=0x100 is the element count (256 F32 values).

## Remaining LFE boundary

The dispatcher later passes the same pointer to two other helpers, which this package does not inspect. Pointer identity to the later e350 call remains static-bound through prior evidence, but the buffer contents after those later helpers and at e350 remain UNBOUND. This does not establish numeric parity, sample-specific behavior, or the cause of the reported correlation.

The upstream Ghidra package manifest SHA-256 is 52604072ce7175b32c60d66b507137955de5ed168b400af255865c4afa5e9d92 and its package-seal file SHA-256 is 1ee4a323de9990acfde2c27a448aa3937b58e8fa3147a6c0a2172018c636013f. Its checker passed normally and under python -O. This supplement excludes raw listings and all media/payload/key material.
