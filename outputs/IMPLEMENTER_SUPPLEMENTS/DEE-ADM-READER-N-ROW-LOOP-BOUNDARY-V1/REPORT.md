# Reader N first row-loop consumer

The selected static Reader path transfers the U64 N field from the Reader record into a Pcm row constructor. That consumer stores N as a Pcm dimension and iterates row pointers while the loop index is less than N. The number of channel rows therefore follows N in this selected call chain. A separate sample-count field controls the byte extent copied for each row (sample_count * 4).

The preceding metadata allocation is guarded by nonzero sample count, not N. Runtime N, the physical CHNA join, owner/generation, and the initializer helper's internal allocation extent remain unobserved. This consumer edge does not establish DEE's full channel-count admission range or a complete ADM-to-output path.
