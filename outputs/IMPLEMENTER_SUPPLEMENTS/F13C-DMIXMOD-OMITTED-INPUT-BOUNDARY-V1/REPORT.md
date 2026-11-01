# F13C downmix-mode omitted-input boundary

## Result

For the hash-bound DME model state, `PreferredStereoDownmix` is inactive. The request builder emits `preferred_downmix_mode` only when the property is active; inactive means the field is omitted. The UI label `Stereo` is not emitted while inactive and is not evidence for a native default.

Ghidra 12.2 shows the native config parser looks up `dmixmod` as a separate optional field. When its presence flag is false, the validator skips choice-map lookup and the invalid-value record path, then returns. The allowed enum strings are `not_indicated`, `pro_logic`, `stereo`, and `pro_logic_ii`; the map order does not prove which value initializes runtime state. No direct alias from `preferred_downmix_mode` to `dmixmod` or consumer edge into the final F13C writer is established.

**Implementation consequence:** preserve the inactive preference as omitted/unspecified and do not ask the user for a LoRo/LtRt choice. Do not invent a native enum default from UI text or enum ordinal. The effective value after omission remains unbound and requires a direct owner initialization/consumer trace before specifying a concrete default.

## Evidence identity and scope

Source package: `F13C-dmixmod-omission-default-trace-addendum-v21`, MANIFEST SHA-256 `cfaec056f570eec6d2a1d985bd3544df30eac446f89140ace9fa551448703415`. Its SHA256SUMS entries for ADDENDUM, Ghidra trace JSON/text, script and manifest were independently recomputed and matched.

The native trace is from Ghidra 12.2 and uses project-recorded image SHA-256 `4b4c624d94a5b47d97b62b28073e1bd351b2deeeea039a7ccfaa2af96562c8a7`; the current installed executable was not rehashed. The DME `app.asar` front-end facts are inherited from the hash-bound v20 package; that bundle was not re-read during this native continuation.

No encoder run, PCM, output payload, or private key was accessed. This package establishes the input omission contract only; it does not establish the runtime enum default or final audio effect.
