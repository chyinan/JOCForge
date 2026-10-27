# Prepared job input identity binding

Internal clean application-boundary clarification. This binds the caller's job document, selected RIFF resource and the ADM graph parsed from that exact resource without changing the approved GTC MeasurementReceipt shape or claiming a native receipt ABI.

## Three identities

1. **Audio resource identity**: `input_sha256` is SHA-256 of the exact complete selected RIFF/WAVE resource bytes, from byte0 through the validated RIFF end, and `resource_bytes` is that exact length. It covers the fmt/data and embedded axml/chna/dbmd chunks. A path, filename, file handle, declared source hash or PCM data-only hash is not a substitute.
2. **Job/config identity**: raw UTF-8 job XML is independently SHA-256 hashed. Normalize and fully admit it under F01E. `config_sha256` is SHA-256 of UTF-8 canonical JSON (sorted keys, compact separators, ensure_ascii=false) for the object in IDENTITY-MAP.json, including raw job XML SHA, selected input label and the complete normalized selected configuration. This is a local identity definition, not a product-visible new MeasurementReceipt field.
3. **ADM graph provenance**: parse AXML/CHNA/DBMD spans from the same immutable RIFF resource whose `input_sha256` is above. Store the resource identity reference with the parsed graph. Chunk offsets/lengths and optional per-chunk SHA are provenance aids; no separately supplied graph/cache may be combined with a different resource. Graph/event admission remains F01E/approved graph rules.

## Resource reuse

Create one caller-owned immutable resource lease/snapshot after RIFF validation and hashing. The parser, meter prepass and main reader all consume it by explicit offsets. Their `ResourceIdentity` values must match the same whole-resource SHA and byte length; `ParsedAdmGraph.parent_resource_identity` and each derived chunk span must point to that identity. If an immutable lease cannot be held, rehash/revalidate the resource immediately before the next phase and reject any mismatch. Do not reopen a path and assume it names unchanged bytes. Do not use the authorized test WAV hash as a runtime whitelist.

The raw job XML SHA and F01E normalized config SHA remain separate from the resource hash. The `config_sha256` canonical envelope makes both caller document identity and its normalized selected meaning visible to the immutable request. The parsed graph is derived from the hashed resource and travels in the same `PreparedJobInput`; it need not be inserted as an extra GTC receipt property or separately folded into `input_sha256`.

## Admission order and scope

Validate XML/F01E selected configuration and caller overrides, validate RIFF shape/descriptor and complete chunk boundaries, hash/acquire the immutable resource, derive the embedded AXML/CHNA/DBMD spans and graph from that resource, then produce the immutable prepared request. Hold/reuse metadata projections only after their approved structural preconditions pass. The earlier worker's cache omission is not product permission to skip admission.

This identity contract does not imply event graph validity from a matching hash; F01E graph, DBMD and single-event/full-duration checks still run. It does not establish that PCM or meter DSP ran, that meter and main outputs match, or that an end-to-end product file was published. Production MeasurementReceipt retains its exact approved fields: `input_sha256` binds this resource, `config_sha256` binds the canonical job/config envelope, and `meter_terminal_sha256` independently binds the finalized meter numeric terminal before the receipt is inserted.

All concrete Rust names/wrappers are LOCAL_TRANSLATION. No additional persistent digest field, root, provider or native ABI is required. The authorized validation input hash is custody evidence for tests only. Historical primary PCM equivalence remains separately NOT_REHASHED and must not be conflated with this new request's actual identity.
