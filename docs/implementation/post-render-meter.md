# Post-render meter callback composition

Status: `LOCALLY_VERIFIED_SYNTHETIC_OAR_PACKET_TO_NUMERIC_LEAVES__FRONTEND_AND_METER_ROOT_DEFERRED`.

`stage_independent_meter_callback` delays the current meter LFE frame by two callbacks, joins it with fifteen caller-supplied same-callback regular SCP rows and control52 rows, then stages the OAR packet through shared six-lane gain delay, publication queue, true-peak block, K-weighted energy window, integrated meter, and ungated mean. One proposal carries the LFE ring and all post-render after values. No leaf is installed during staging.

The callback index and upstream generation must agree across OAR, DV32, and the meter queue. The queue retains actual sample words for every lane. Each complete 1,024-sample block advances true peak and K/window state once. Each completed 960-sample energy record advances integrated and ungated state once.

The local composition vector uses synthetic regular rows and controls, the selected direct control row, and positive-zero PCM. For one source sample, one normal callback and three zero-input drains produce 1,536 published samples per lane, one complete block, 512 raw samples pending, one energy record, and 64 samples in the next energy window. The separate seven-frame ring recipe verifies that output `g` comes from input `g-2` after priming.

These vectors verify the typed post-render subchain and the direct-LFE-to-OAR packet join. They do not produce regular SCP/control from an independent ADM frontend, establish a whole-meter terminal identity, create a `ProgrammeMeasurementReceipt`, or supply the lineage-bearing ID11/ID2/ID14 bodies that `MetadataOwner::stage_source` requires. The existing metadata source stage can consume a verified measurement receipt; this callback proposal is not that receipt.
