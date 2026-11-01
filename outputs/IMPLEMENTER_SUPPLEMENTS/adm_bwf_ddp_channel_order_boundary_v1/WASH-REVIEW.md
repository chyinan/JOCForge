# Wash review: ADM BWF to DDP channel-row boundary

This bounded Ghidra trace establishes two separate facts: the converter uses CHNA-derived track indices to construct its local ADM channel association; the first DDP row-table consumer copies row `i` to internal row `i`. It does not connect the reader's PCM row ordinal to that DDP input ordinal, and it does not establish whether a permutation occurs before the DDP copy.

Do not infer identity order or a CHNA-driven permutation from the matching 38-channel count or equal ordinal values. The channel mapping affects audio. It therefore remains DEE-derived/approved-contract behavior. The single minimum closure observation is a same-job/generation metadata-only correspondence across ADM identity, CHNA track number, reader output row, and DDP input row; PCM sample data is not needed.
