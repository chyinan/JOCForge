# F13C preferred downmix input boundary

A metadata-only scan of the seven authorized ADM BWF inputs found no downmix, LoRo, LtRt, or preferred terms in their axml chunks. Independent Wash-side RIFF scans reproduced the same result from the axml bytes while seeking over the data chunk. No PCM was read, decoded, copied, or hashed.

The inspected DME DDP online-media UI profile treats PreferredStereoDownmix as optional: it is inactive at initial state, and its request builder emits the parameter only for active controls. The initial display value Stereo is not automatically sent. When activated, the UI maps Stereo to loro, Surround to ltrt, and Not Indicated to not_indicated. Four mix-level controls are separate and are also inactive initially.

Native static observations report a separate dmixmod key and four separate mix-level keys; an absent-field path skips parsing. The native default after omission, the DME preferred_downmix_mode to dmixmod adapter, and the final F13C writer/consumer branch remain UNBOUND. This package does not establish a codec output value or require a new user prompt.

Source caveat: the research manifest did not compute the DME app.asar or native encoder executable hashes. Wash computed hashes for the current installed files, but cannot use those post-hoc hashes to identify the exact files imported into the earlier Ghidra/UI analysis.
