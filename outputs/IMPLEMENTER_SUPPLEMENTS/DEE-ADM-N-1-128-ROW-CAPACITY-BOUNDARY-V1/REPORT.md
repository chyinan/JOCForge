# ADM physical N gate and first row capacity

The selected physical-N path has a static unsigned gate: fail with local code 5 when N-1 exceeds 0x7f. This bounds that gate's accepted input to N=1..128; N=0 underflows the unsigned subtraction and N=129 crosses the upper boundary. Runtime acceptance was not run.

The first row wrapper has a fixed 128-slot capacity and constructs 128 per-slot objects. It receives a row-map buffer and construction arena, not a separate N argument. The later per-slot row count remains a distinct domain. One constructor receives numeric parameters 0x600, 1, and 48000; the meaning of 0x600 is not inferred.

K and composite B are separate count domains. K is an individual-record count plus group-entry counts, stored as low U16 from a 32-bit accumulator, with six-byte rows. B follows a separate adapter/UID-derived path and allocates B_count * 6. The physical N gate, fixed wrapper capacity, per-slot count, K, and B have no typed N→K/B join here. Native K/B values and runtime mode remain uncaptured; 36/38 samples are not used to infer this range.
