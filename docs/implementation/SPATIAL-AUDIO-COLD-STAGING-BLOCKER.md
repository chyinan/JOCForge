# P1.3-E1 cold-state/staging blocker

## Decision

`P1.3-E1` is blocked before Rust implementation. The approved package does not uniquely determine every scalar in the complete cold preimage or all byte-level rules needed to reproduce its digest. Do not create `SpatialAudioState`, a machine, or staging tokens; do not substitute guessed values or treat the expected digest as a recoverable preimage.

The machine-readable task status is `BLOCKED_ON_COLD_STATE_SPEC_DETAILS`. `P1.3-E` remains `BLOCKED_ON_F04_PROVIDERS_RENDER_AND_PARENT`; the separate cold-state blockers below must also be resolved before E1 can start. P1.3-C remains waiting for owner roots.

## Pinned evidence

Approved package manifest SHA-256: `c7cddb7d5ac073c0f410fe44be9a431cfbb21c3df469d8b43c32a4297298674a`.

| Evidence | Identity | Relevant content |
|---|---|---|
| `test-vectors/F04-WHOLE-SPATIAL-STATE-SCHEMA.json` | 226344 bytes; SHA-256 `4fdab9f591821d7c50e7daca17bc98d4a5b0616e9cb45b5937cae47b26ed657c` | Cold schema pointer `#/whole_state_by_boundary/cold`; nine ordered `F04CompositionState` fields and recursive types. `TemporalState` contains `span`, `stride_bytes`, `index_28`, `index_32`, `buffers`. |
| `test-vectors/F04-WHOLE-SPATIAL-CHAIN-VERIFICATION.json` | 260481 bytes; SHA-256 `ffb1dea3b1e69383312da4059b9d8ddabb4366ed8fc22e7537f750ba041f2d63` | `reset_check.cold_state_sha256 = 35cfac927bbdca64f28954844938fe2936531f59f049dfca911c44d3a38b0b5e`; this is a digest, not the canonical byte preimage. |
| `test-vectors/QMF-RECEIVER-WHOLE-BINDING-VERIFICATION.json` | 74454 bytes; SHA-256 `1d809ed24c805a182e9f2ba668562f1f3c8028e4dfc0ce7cd039e1255e4cda07` | Recursive `ModelState` / `QmfWindowState` schema and component digests; input cases carry before/after hashes, not raw cold state values. |
| `specs/04-receiver-state-and-routing.md` | 8332 bytes; SHA-256 `74b9b01537eb76a37582535b84426138ee8c8ecdca3486f9f43d0504c7c26306` | F04R.1 gives temporal spans, zero buffers and semantic `read=span/write=0`; it does not bind those semantics to `index_28` vs `index_32`, or give `stride_bytes`. |
| `specs/04-qmf-receiver-whole-binding.md` | 13595 bytes; SHA-256 `104d604cf876cc522a85b21f352bd0a09fc20c3171ddcd71da981c8076ea3a75` | F04RB.7 gives the exact recursive types. It identifies `stride_bytes/index_28/index_32` as integers but does not provide their cold values. F04RB.6 names target temporal span/buffers but not those scalar values. |
| `specs/04-whole-spatial-chain-composition.md` | 9103 bytes; SHA-256 `92325b12128023001e1f3fd4696dcec68bbd47a46acdb7886781b411a10c5d59` | F04C.5 summarizes the complete cold state and references the schema; it does not add the missing temporal scalar binding. |

## Exact missing bindings

1. For `receiver_state.temporal_states[0..5]`, the package supplies spans `1217, 63, 900, 256, 896` and buffer shapes `T3<B9736>, T5<B504>, T1<B7200>, T6<B2048>, T15<B7168>`. It does not give exact cold `stride_bytes` values or explicitly map `index_28` and `index_32` to the semantic read/write values. The F04R.1 phrase `read=span/write=0` is insufficient to serialize the named fields without guessing.
2. For `fitting_qmf_state.target_temporal`, the package supplies span 896 and `T15<B7168>` buffers but likewise does not give exact `stride_bytes/index_28/index_32` cold values or the semantic index-field crosswalk.
3. `F04-WHOLE-SPATIAL-STATE-SCHEMA.json` identifies `F04_STATE_NODE_V1` tags as `N/B+U64LE/I+I64LE/F+F32LE/Z+U8/T+U64LE/D+class+declared fields`, but does not specify the byte lengths/encoding of the D class name, field count, or field names. F04RB.7 defines U16LE lengths for `encode_node-v1` over its QMF/receiver state scope; the approved package does not explicitly bind that D-header rule to the complete `F04CompositionState` tree or identify the two encodings as the same whole-root serializer.

The required wash-side supplement/erratum should give a complete constructor table for all six temporal nodes (including the index crosswalk and stride values) and the complete `F04_STATE_NODE_V1` byte grammar for every node class. A literal canonical cold byte preimage or an independent cold-serialization known-answer vector would make the digest binding directly verifiable. Until then, no zero/default value may be substituted, no GTC cold digest may be claimed as reconstructed, and no local cold storage/staging implementation may be marked complete.

## Scope preserved

No DSP, F04 provider validation, PostWindowAudioPacket producer/fanout, or local commit/abort/fatal API is authorized by this record. The established F04 owner split, `None` sentinels, callback/receiver clock separation, and GTC-only commit authority remain unchanged.
