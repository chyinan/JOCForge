# F12 grouped quantizer local slice

This package is an internal-only, bounded helper contract for valid BAP values 1, 2 and 4. It covers quantized coefficient dispatch, grouped packing, tail fill and incremental word-writer state.

Static rules:

- BAP1: groups of three, radix `9*x0 + 3*x1 + x2`, width 5, fill code 1.
- BAP2: groups of three, radix `25*x0 + 5*x1 + x2`, width 7, fill code 2.
- BAP4: groups of two, radix `11*x0 + x1`, width 7, single-item fill code 5.
- Flush clears group counters and returns zero.

Runtime-confirmed vectors cover finite values, NaN/infinity handling observed by the native helper, finite extremes, mixed incomplete groups, empty flush, and a logical-capacity canary case. Hashes for emitted word-memory bytes are included in the vector files; they are not complete EC3 output hashes.

The helper has no capacity argument, status result, or rollback. It mutates reservation and group state incrementally; caller-owned capacity and higher-level retry are outside this package.

Excluded:

- BAP values outside 1/2/4;
- upstream source-plane/producer identity;
- full codec frame/grouping integration;
- caller capacity ownership and retry/rollback;
- real audio, PCM/ADM, private keys and research binaries.

`production_usable=SLICE_ONLY` and whole-product `IMPLEMENTATION_READY=NO`.

