# Human Cat DRC plugin/core F32 row ownership boundary

## Plugin-local row

The Atmos plugin image contains a local context whose own constructor zeroes 30 U32 words beginning at local `+0x2170`. A plugin reader consumes F32 values from the corresponding six-by-five row. The same local context also has a separate max/history path that reads from `+0x168` and updates `+0x2168`; it does not write the six-by-five row. A whole-image direct-displacement scan found no additional direct nonzero writer to this local row, while alias-mediated and bulk-copy paths remain outside that scan's scope.

## Core row remains separate

The E-AC-3 encoder core has a different inline state object at `state+0x21cc4`; its constructor zeroes a six-by-five F32 row and its profile producer reads that row. This package found no typed host/API copy or owner edge from the plugin-local context to the core inline context. Matching offsets, row sizes, and reader shapes across separate images do not establish shared identity or value flow.

The core row's generation-time writer is still unbound. Do not use the plugin-local zero initializer or history reducer as the core row's producer. The next edge is a typed writer/update into the core inline row, including a plugin-to-core copy only if directly traced.

No PCM, encoded payload, private key, license material, native source code, or disassembly is included.
