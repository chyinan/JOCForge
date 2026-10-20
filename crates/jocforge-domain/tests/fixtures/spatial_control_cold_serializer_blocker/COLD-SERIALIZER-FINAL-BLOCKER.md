# Spatial-control cold serializer final blocker

## Result

EXECUTABLE_AUTHORITATIVE_COLD_SERIALIZER=NOT_FOUND_IN_AUTHORIZED_EVIDENCE

SERIALIZER_EXECUTION=NOT_RUN

CANONICAL_PREIMAGE=NOT_DELIVERED

SECTION_OFFSETS=NOT_DELIVERED

AUTHORITATIVE_LENGTH=null

AUTHORITATIVE_SHA256=null

BYTES_GUESSED=false

AI48ST01_USED=false

No production success receipt is generated.

## Executable entrypoint audit

The pinned dee_audio_filter_ddp_atmos.dll exports only the Phoenix audio-filter
handle and version handle. No serializer, cold-state or snapshot export exists.
No imported symbol matching serializer/state/cold/snapshot was found.

The IDA database decompiles the parent callback at RVA 0x1A7710 and records its
thirteen direct calls. Five bounded frontend candidates at RVAs 0x1AA8A0,
0x1AB800, 0x1AC490, 0x1ACD10 and 0x1AD9D0 are runtime processing, descriptor,
control writer or receiver functions. None serializes the selected parent
state. The parent call map contains no serializer call. The DLL contains no
AI48ST01, selected-state-schema or F01E cold magic string.

The expanded audit scanned 64 PE files in the authorized DEE binary suite.
Only four MediaInfo state-query exports matched broad state-related naming;
they are unrelated to spatial control serialization. No serializer, snapshot,
cold or persistence export and no schema magic was found.

Therefore no minimum cold-constructor-only invocation can be specified from
the authorized executable evidence. This does not prove that no internal
serialization could ever be written; it proves that no executable
authoritative serializer entrypoint or call path is available in the admitted
evidence.

## Known constructors and layout

The F01E nested cold object has an exact 4402-byte sorted compact UTF-8 JSON
preimage and SHA-256
9c943879cd36cb45bc7538346cfdf4f71d0998d8f261f75c135045fcb59870aa.
The selected target cold constructor, provider manifest and parent layout
schema are pinned. Parent field order is clocks, source_cache, event_control,
render_audio, source_flags, feature, selection_outer and matrix_assignment.

Those facts do not uniquely produce parent bytes.

## Exact missing framing rules

### Top-level envelope/version

Missing: magic/version, total-length field, section identifiers, section
length widths, alignment and terminal rule. Field order alone is insufficient.

### F01E embedding

Missing: whether the 4402-byte canonical JSON is embedded raw, length-prefixed,
converted to a binary projection, referenced by digest, or wrapped with a
presence/version tag. The exact F01E bytes cannot simply be concatenated.

### Optional diffusion/source-cache

Missing: exact absence representation, presence-bit grouping, ordering of
optional last host/control snapshots, diffusion-instance records and optional
diffusion previous-output arrays. Cold absence does not define framing.
The generic optional-presence U8 rule and cold absence are frozen facts, so
this item is partially defined rather than wholly unknown.

### current_flags/current_features persistence

The schema calls current_flags and current_features callback outputs with no
independent history, while also listing source_flags/feature under the parent
owner. Missing: whether cold serialization excludes them, includes zero arrays
or stores an optional callback snapshot.

### Writer-set framing

Cold writer-set is empty. Missing: record field order/types, count/length
framing and whether destinations/owner/generation tokens are inline or
referenced. An empty set does not reveal its record encoding.
The generic U32LE count/length rule establishes an empty logical count, but not
the containing section or nonempty record schema.

## Minimum external evidence

Either provide an executable authoritative serializer, or specify every
missing framing rule above. Then run the cold constructor only and deliver
complete preimage, per-section offset/length, total length and SHA-256.
An independent parser must round-trip every field and prove that no scratch,
pointer, structure padding or unknown zero-fill is present.

Spatial render, selector, U04 and outer are not required and remain NOT_RUN.
