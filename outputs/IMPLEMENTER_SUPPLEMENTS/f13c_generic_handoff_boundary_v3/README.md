# F13C generic configuration handoff boundary

The hash-bound Ghidra trace confirms that the native parser reads dmixmod as a distinct enum field and routes it through its enum converter. On the parser-success branch, the caller passes configuration result objects to a generic handoff routine. That routine iterates 0x68-byte records and builds name/value/error entries; the targeted body did not show downmix-key reads or a field-specific consumer.

This establishes a parser-to-generic-result handoff only. It does not identify those records as F13C payload data and does not connect dmixmod to a DDP JOC/EC3 writer, stereo postmix, or target 5.1 signaling. Do not infer that dmixmod has no effect; the consumer edge remains UNBOUND.

The imported Ghidra image hash matches the current installed executable hash rechecked by Wash. The prior DME UI observation remains contextual only; neither the preferred_downmix_mode adapter nor the native omitted-field default is established. No encoder, PCM, payload, or key was used.
