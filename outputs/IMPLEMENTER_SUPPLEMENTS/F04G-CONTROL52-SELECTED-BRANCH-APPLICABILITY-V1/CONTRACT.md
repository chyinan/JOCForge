# Control52 selector branch applicability

This supplement distinguishes the approved clean ADM profile fields from the native context selector fields. It does not infer one field from another.

The approved F01E contract defines `warm_trim_mode` and `authored_source_mode_byte = warm_trim_mode + 1`; for its selected input, the byte is 4 and is used to derive `surround_ex_mode`. The same contract explicitly treats these as separate configuration domains. It does not define that byte as the native context's `source_mode` or context mode flag.

The approved ADM graph/event contract selects 38 PCM channels, ten beds, 28 objects, and the defined event shapes. Bed/object physical binding uses `channel_format_id`, semantic role, object ordinal, and `host_channel_index`. It does not provide a crosswalk from those clean identities to the native selector builder's category byte or its supported-category set.

Therefore the static native condition `source_mode != 1` plus an unsupported in-range category remains conditional. The selected profile cannot be declared to take or exclude that branch from the clean mode byte, bed/object counts, or equal numeric values. Keep selected branch applicability `NOT_OBSERVED` until a direct approved identity crosswalk exists. The formatter-to-selected-gate owner/generation join also remains `UNBOUND`.
