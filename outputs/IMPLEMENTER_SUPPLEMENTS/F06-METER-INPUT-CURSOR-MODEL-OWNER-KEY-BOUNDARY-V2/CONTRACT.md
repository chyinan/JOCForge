# F06 model owner-key boundary, V2

This supplement supersedes the V1 callsite status. It closes the frozen model API and the limited frozen-callsite search; it does not identify a production Meter owner-key producer.

model_owner_key is an ordinary owner identity string, not a vendor private key or authentication credential. The frozen model requires a non-empty Python string, defines no grammar or derivation formula, and uses it as a map key. Map hits check stored key and duration, but the API does not compare resource SHA, source-instance token or attempt. V4 copies a supplied recipe value into the local root and separately binds source_identity_sha256 to binding.input_sha256. It does not define key generation or attempt+1 renewal.

A search of the frozen execution-sources tree found two direct cursor-initializer callers, both in Panner numerical models. They supply a fixed declared-storage label and len(wave.data); neither supplies input hash, source-instance, attempt or Meter-root token. No external Meter-root caller of read_pcm_source_owner or complete_pcm_read_schedule was found. These callers are not Meter ingress.

The selected Meter key producer remains UNBOUND. The DLL identity is rehashed but native static analysis is NOT_RUN because this session has no usable IDA/Ghidra interface. Minimum evidence is a metadata-only receipt tying admitted input identity/extent/format/attempt, opaque storage owner, key producer/call token, cursor map/read receipt, and the same Meter root/parent generation through commit. Do not infer the production key from Panner constants or synthetic V4 examples.
