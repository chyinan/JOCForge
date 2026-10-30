# Table-B field identity review hold

The prior Wash supplement established a native SpatialCodingState table-B producer/consumer path and kept its F06OF target-slot/control-word join `UNBOUND`. One narrower field claim is now under re-review: the prior summary described the U32 read from an ordered-map node as a value and said the map key was dropped. Agent 3 reports that the current instruction view does not yet distinguish whether node offset +0x1c is the key or value; allocator/insert/comparator layout is being traced.

Until a hash-bound correction is reviewed, do not rely on either “the key is dropped” or “this U32 is the value” as a Wash-verified fact. Keep the U32 field's role, source semantic key, and any F06OF crosswalk `UNBOUND`. The broader native owner/consumer path remains bounded evidence. No row-count, stride, index, or scalar-equality inference is permitted.

This is a provisional evidence-status hold, not a new product rule. The investigation continues; production join and whole-chain readiness remain unverified.
