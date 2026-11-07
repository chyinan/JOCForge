# DEE physical N gate and count-domain capacities

The selected physical-N gate applies an unsigned check that fails with local code 5 when N-1 exceeds 0x7f. Its static accepted range is 1 through 128. It writes N into the map owner and loops over N source descriptors. Runtime acceptance was not run.

The first row wrapper has 128 slots and constructs 128 per-slot objects; it does not receive N as its allocation count. A later reader also loops over all 128 map slots and consults a separate per-slot row-record count. One type-1 factory receives parameters 0x600, 1, and 48000; the meaning of 0x600 is not inferred.

K is a separate count: individual records plus group entries, accumulated at 32 bits and narrowed to U16 at K+0x0A. K rows start at K+0x10 and use six-byte strides. Composite B has separate ordinary and extended formulas, both allocating B_count * 6; the selected mode and native values were not captured. Physical N has no typed join to K, B, or per-slot row counts in this evidence. The 36/38 projections remain input-only examples, not runtime range evidence.
