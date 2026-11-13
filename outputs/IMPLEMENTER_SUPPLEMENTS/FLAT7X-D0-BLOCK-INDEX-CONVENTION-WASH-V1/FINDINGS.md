# Flat7x D0 block-index convention

## Resolution

For the approved Flat7x target, “D0 block 2” means zero-based `block_index == 2`. It is the third audio-block prefix in that D0 syncframe. The target evidence package stores `d0_emdf_block_index: 2`; the current Wash handoff explicitly describes the approved field as a “zero-based block-2 skip field.”

OpenJOC uses the same convention: `AudioBlockCarrier.block_index` is documented as zero-based, the walker iterates `0..audio_blocks`, and it assigns the loop index to the callback record carrying that block's `skip_field`. This number is the parser's sequential position among audio-block prefixes after the frame header/BSI; the evidence does not identify a separate per-block ordinal field in BSI.

The earlier official-sample receipt selected `block_index == 2` and labels it human ordinal 3. That is the same zero-based slot specified by the approved target contract. The locator sample remains 640 kbps; this reconciliation says nothing about 1024-kbps payload behavior.

## Boundary

The numbering convention is resolved. Native D0 insertion ownership, audio-source generation, and production output retain their existing `UNBOUND` / `NOT_RUN` status.
