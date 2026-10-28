# F06 model owner-key boundary

This wash closes the frozen Python model field semantics and V4 local-root projection only; it does not recover a native/DEE owner-key producer.

The field is an ordinary model owner identity string, not a vendor private key or authentication credential. The frozen model requires a non-empty Python string and defines no length, charset, prefix, digest syntax, path form or derivation formula. The owner map uses the supplied string as its lookup key: a miss constructs a cold cursor; a hit checks stored key and duration before transition/writeback. It does not compare a resource SHA or attempt in the map API. Reusing a key can select an existing cursor; collision avoidance is a caller responsibility, not a reported collision.

For 38-channel PCM24, a frame is 114 bytes. The model requires positive U64 byte length divisible by 114 and initializes duration=data_length_bytes/114, source_cursor=0 and read_index=0.

V4 places the cursor inside owners.meter, ties source_identity_sha256 to binding.input_sha256, and copies a supplied source_owner_key into the synthetic root. Full restart destroys owners and increments attempt, but neither model nor V4 defines key reuse/renewal on attempt+1.

Production edge remains UNBOUND: admitted input/storage instance and attempt -> owner-key producer/call token -> cursor map slot/read receipt -> same meter root -> parent commit. DLL identity is rehashed, but native static analysis/execution is NOT_RUN because no usable IDA/Ghidra interface was available. Do not infer a key formula from synthetic examples.
