# Reader N Pcm allocation arithmetic boundary

The helper multiplies Reader N by sample_count and uses the low 64-bit result as an element count. It compares that count with the stored U32 capacity at object +0x48. If the buffer must grow, it releases the old pointer at +0x40 and requests four bytes per element. The helper does not check overflow of N × sample_count. It checks the high word of the later 128-bit byte multiplication; on overflow, it passes UINT64_MAX to the allocator. No explicit allocator failure branch was observed.

If S is nonnegative and the conditional N gate 1..128 applies, an exact byte extent fits U64 only when S ≤ floor((2^64−1)/(4N)). The bound for every N in that range is 2^55−1. This is an arithmetic bound only. Runtime N, S, capacity, and allocation outcome were not captured; no PCM was read.
