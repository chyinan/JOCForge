# Reader N and sample-count argument boundary

The selected Reader call chain carries two separate inputs into the PCM allocation helper. N comes from the Reader descriptor's U16 channel-count field at +0x0A and is zero-extended into the Reader record. Sample count S comes from a different path: depending on Reader-state byte +0x14, it is either the float-read result count or a U32 Reader-state field at +0x10.

The Reader record forwards N and S as separate constructor arguments; the constructor forwards each to its corresponding allocation-helper parameter. Runtime values, the selected S branch, and allocator results were not captured.

The typed argument path is bound within this Reader call chain. The physical CHNA N to this Reader descriptor join, and this Reader descriptor to the separately copied mode-zero composite K descriptor under one owner/generation, remain UNBOUND. This does not establish an end-to-end channel-count range. No DEE run or PCM/payload/key/license access occurred.
