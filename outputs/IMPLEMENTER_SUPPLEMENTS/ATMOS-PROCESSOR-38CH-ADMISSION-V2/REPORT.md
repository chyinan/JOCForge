# AtmosProcessor 38-channel admission and capacity

The hash-bound Atmos plugin has no fixed 36-channel cap in the traced process
guard. The selected master descriptor stores the configured channel count as
U16 at `+0x0A`; the reader writes that same value to descriptor `+0x08` and
passes the same master pointer at descriptor `+0x28`. Processor initialization
reads master `+0x0A` into state slot 4. For nonzero frames,
`AtmosProcessor::process` requires descriptor channel count to equal state
slot 4. Thus 38 channels pass this guard when the selected instance's master
U16 is 38.

The reader's count is written at `CompositeMaster+0x1A` from either the first
child source count or the special rule `10 + max(object-marked records)`. For
the authorized “人是猫” metadata (10 bed tracks and 28 object tracks), either
an ordinary child count of 38 or 28 matching object markers can yield 38.
The BWF fmt field is not directly read by this traced constructor chain; its
runtime join to a child count or marker set remains open.

The selected profile uses 1536-frame blocks. Initialization accepts 512 or
1536; zero frames are flushes only after initialization. The static guard
checks frame count against the configured state capacity.

This is a strong static admission contract, not a 38-channel DEE run. The
specific 人是猫 runtime `master_descriptor+0x0A` value is still unobserved,
and no complete output or audio playback is claimed. Wash contains no raw
decompilation or binary.
