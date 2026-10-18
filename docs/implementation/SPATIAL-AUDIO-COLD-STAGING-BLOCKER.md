# P1.3-E1 cold-state/staging blocker

> Status: `RESOLVED_BY_F04_COLD_SUPPLEMENT` for specification completeness. Local typed construction, serialization, and post-construction KAT comparison remain pending P1.3-E1.

## Decision

`P1.3-E1` was blocked before Rust implementation because the original approved package did not uniquely bind every temporal scalar or the complete D-node grammar. The wash-side supplement below resolves those specification gaps. The expected digest is still not a recoverable preimage: implementation must construct typed state and serialize it before comparing bytes.

The previous machine-readable blocker status `BLOCKED_ON_COLD_STATE_SPEC_DETAILS` is now `RESOLVED_BY_F04_COLD_SUPPLEMENT`. `P1.3-E` remains `BLOCKED_ON_F04_PROVIDERS_RENDER_AND_PARENT`; P1.3-C remains waiting for owner roots.

## Resolution supplement

The implementation supplement identity is `f04-spatial-audio-cold-state.v1`; its manifest is 2,905 bytes with SHA-256 `77ad0562fcd4efa1e8fe37cc1dc7219570b49b9b332cca1f1273ccda2e2cef9c`. All 12 manifest file identities were checked. No `research_basis` files were opened. `F04-COLD-PREIMAGE.bin` was streamed only for manifest identity hashing; its contents were not used to build state.

The supplement pins the expected cold output to 813,733 bytes and SHA-256 `35cfac927bbdca64f28954844938fe2936531f59f049dfca911c44d3a38b0b5e`. The KAT remains a post-construction comparison target.

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

## Previously missing bindings, now resolved

1. For `receiver_state.temporal_states[0..4]` and `fitting_qmf_state.target_temporal`, the supplement pins `stride_bytes=4`, `index_28=read=span`, and `index_32=write=0`. Each buffer tuple is lane-major; each lane contains `2*span` F32 positive-zero words as opaque raw bytes. The two span-896 nodes have equal content but own separate buffers.
2. The supplement defines the complete recursive `F04_STATE_NODE_V1` grammar: `N`, `B`, `I`, `F`, `Z`, `T`, and `D`; all widths, endian rules, ASCII naming, declared field order, type precedence, and exclusions are explicit. D nodes use U16LE class/name lengths and declared field count, with no D body length.

The supplement provides the six-node constructor table and complete byte grammar. Implementation must still build a fresh typed nine-field root, run the project-owned encoder, and only then compare the generated 813,733 bytes with the KAT. It must preserve `None`, bool, signed integer, float, and raw bytes as distinct kinds; keep both span-896 temporal states independent; reject a negative-zero cold float; and never return KAT bytes as encoder output.

## Scope preserved

No DSP, F04 provider validation, PostWindowAudioPacket producer/fanout, or local commit/abort/fatal API is authorized by this record. The established F04 owner split, `None` sentinels, callback/receiver clock separation, and GTC-only commit authority remain unchanged.
