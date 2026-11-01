# Wash review: DDP samples alias and cross-image boundary

Within one hash-bound DDP image, the source metadata map is copied to a local map. If one or more of `channels`, `sampling_rate`, or `samples` is missing, the path conditionally maps MediaInfo keys `Channels`, `SamplingRate`, and `SamplingCount` into the local map. The caller copies local `samples` to an output map under the same DDP context; a later DDP timeline consumer reads it.

This is a conditional field-name/value alias path, not an audio algorithm. It does not prove what value populated an already complete source map, how MediaInfo computes `SamplingCount`, or that DDP's map is the AtmosMezz file-position converter's map. The converter has one literal `samples` key consumer, but no typed owner/pointer join to DDP or to the reader extent. A similar key or numerically equal value does not establish identity.

The earlier sample-6 BWF header calculation (`data_chunk_bytes / block_align = 6,522,906`) was not used to assert this DEE producer. For JOCForge, direct header parsing remains an independent I/O choice; do not label it as DEE's hidden source rule. A same-job DDP-maq’AtmosMezz-map’reader scalar observation is the only proposed dynamic join.
