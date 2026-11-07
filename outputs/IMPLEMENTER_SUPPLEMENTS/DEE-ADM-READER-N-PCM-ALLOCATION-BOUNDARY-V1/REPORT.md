# Reader N Pcm row allocation boundary

The selected mode-zero initializer stores N and sample_count separately. It requires N × sample_count 32-bit elements. When the stored capacity is smaller, it releases the old buffer, requests 4 × (N × sample_count) bytes, and records the element capacity. It builds N row pointers with a 4 × sample_count byte stride.

The byte-scaling multiplication checks the high word of a 128-bit 4 × (N × sample_count) product. On overflow it passes UINT64_MAX to operator_new. The earlier N × sample_count product has no explicit overflow guard in the inspected helper. Runtime N, sample_count, existing capacity, and allocator result were not captured; no PCM was read. This package does not establish complete DEE admission or runtime acceptance for the full 1..128 range.
