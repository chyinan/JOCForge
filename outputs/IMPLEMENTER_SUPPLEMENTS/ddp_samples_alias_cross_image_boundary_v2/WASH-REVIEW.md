# Wash review: DDP samples alias (V2)

In the hash-bound DDP image, the source metadata map is copied to a local map. If `channels`, `sampling_rate`, or `samples` is missing, a conditional fallback maps MediaInfo `Channels`, `SamplingRate`, and `SamplingCount` to those keys. The caller copies the resulting local `samples` value into the output map of the same DDP context, and a later DDP timeline routine reads it.

A complete preexisting input map bypasses the fallback, so its writer remains unbound. The analysis does not establish MediaInfo's SamplingCount formula or a `data_chunk_bytes / block_align` calculation. It also does not join the DDP maps to the AtmosMezz converter's separate `samples` map or the opened reader. Matching key names or equal values do not establish identity.

This is a conditional metadata crosswalk only, not an audio algorithm. The single remaining closure observation is a same-job/generation capture connecting the DDP map branch, AtmosMezz `samples` read, and actual reader extent/EOF.
