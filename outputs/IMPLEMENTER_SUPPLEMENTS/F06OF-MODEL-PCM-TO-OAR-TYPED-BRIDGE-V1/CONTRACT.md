# Meter PCM source to OAR typed bridge

Ready scope: the selected model's input/count/row adapter and same-callback handoff through the existing frontend, spatial/component, LFE, OAR and shared-DV providers. This authorizes connecting those specified stages; it does not certify a new DSP run or native pointer identity.

## Input identity and two count domains

Decode PCM24 into 38 planar physical-channel rows. Attach ADM metadata through approved F01 UID resolution and physical_slot=track_index-1; CHNA entry order does not reorder audio. Legacy ordinal fields used to index source rows must be populated by this typed physical-slot adapter, not by enumeration or equal row counts.

On a normal callback preserve old_cursor, obtain A=min(1536,N-old_cursor), decode exactly A samples per row, and advance the source cursor by A. The address is payloadOffset+(old_cursor+relative_sample)*114+3*physical_channel. Each decoded F32 row has A*4 bytes; append fresh positive-zero raw input tail to reach 1536 samples. Keep clean host_input_count=A and frontend_samples=1536 distinct. N=0 is rejected by the selected count admission.

The meter has its own cold cursor and processing states. After Q=ceil(N/1536) normal callbacks, issue three new zero-input callbacks with raw38 input, host_input_count=0, frontend_samples=1536 and cursor=N. These advance the existing meter processing states; they do not read the source again, reuse the main encoder's cached drains, or clear derived filter/audio tails.

## Same-callback derived rows and controls

Pass the same padded normal/drain input and its typed metadata to the existing independent meter frontend/spatial providers. Their matrix/component stage produces fifteen component rows. The semantic direct LFE physical row follows the existing independent LFE path and three-slot ring. For callback g<2 no ring output exists and the allocated LFE row is owned positive zero; for g>=2 the ring output originates at g-2. This formula applies even when Q=1; the shortcut description about two drains returning the last two normal inputs only applies when Q>=2.

Compose the immutable allocated input as [LFE, component0,...,component14] and bind its corresponding regular/direct controls in that same callback. The model producer writes those exact allocated16/control artifacts and the OAR consumer reads those same artifacts and indices. Product code may use the equivalent owned typed in-memory results; it need not reproduce research temporary files. This is a transformed 38->16 dataflow, not a direct pointer alias.

OAR consumes allocated16 on all Q+3 callbacks. Do not substitute the optional post-priming spatial artifact stream, which contains only Q+1 callbacks. Each OAR row has 1536 F32 samples; eight two-row task states consume 48 blocks of 32 samples and retain state across callbacks. The six output rows feed the existing shared-DV provider with its independent carried state.

Regular metadata60 stores 1536 in its frame field. The direct branch retains constructor zero in that field while the process extent is still 1536. These are not the same field/domain as a native U64 descriptor ABI. The current approved canonical direct-control adapter keeps the consumed word2 and initializes only its owned unconsumed fields; this does not claim all native direct52 bytes are known. Preserve the fifteen supplied regular controls and their producer/history provenance.

## Programme output and lifecycle

The startup crop uses latency 3104. Published length is P=Q*1536, including input padding, and statistics consume C=floor(P/1024)*1024. Energy events use complete 960-sample windows within C; leftover 1024 and 960 tails create no extra EOF event. The crop spans are intersections of each full output callback interval with [3104,3104+P).

Callback g is zero-based; batch generation is g+1. History-delivered control payloads keep their original producer row/generation rather than being relabelled by current batch or folder index. Full model restart reconstructs the independent owners and starts callback zero. Use the existing ownership/receipt architecture to carry this causal dataflow; no native Rust receipt name, digest or global transaction is required.

The archived external meter report was unavailable and is not fresh execution evidence. Its absence does not invalidate the exact model producer/consumer rules or the approved F01 adapter. The source package's nine count cases and two virtual row-label cases exercise interfaces only; no actual PCM decoding, frontend/render/OAR DSP or native run was executed by these checks.
