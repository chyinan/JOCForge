# Independent meter LFE delay and OAR packet assembly

Status: `LOCALLY_VERIFIED_TWO_CALLBACK_LFE_RING_AND_PACKET_ASSEMBLY__UPSTREAM_FRONTEND_DEFERRED`.

`MeterLfeDelayState` owns a fresh three-slot ring. Each slot is absent or stores all 1,536 finite sample words and its originating callback index. One staged callback advances the read/write slots once and writes the supplied current LFE frame. The first two callbacks return allocated positive-zero output; later callbacks return the complete frame from `g-2` with its origin.

`stage_meter_frontend_oar_packet` joins that delayed LFE frame with fifteen same-callback regular SCP rows and fifteen complete control52 rows. It writes the selected direct control row, maps meter callback `g` to OAR generation `g+1`, and returns both the ring after state and the immutable OAR packet.

The local recipe replays seven frames: four normal values followed by three zero-input drains. A packet test checks the direct row, regular row identity, phase, and generation. Another combined callback test feeds the packet through the post-render meter proposal.

This boundary starts after upstream processing has produced the current LFE row, regular SCP rows, and regular controls. It does not implement source reading, object rendering, regular component production, or the complete independent meter frontend.
