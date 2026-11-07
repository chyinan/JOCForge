# Current 36-row production-gate order

This package corrects only the current gate order in JOCFORGE-36-ROW-DYNAMIC-SHAPE-COLD-RESET-V1. Keep its 36/38 row-shape and new-attempt reset contract, but do not use its older wording that FreshZero is the first fixed-38 gate.

## Current worktree at HEAD aefd8cc

1. WAVE/ADM admission is dynamic for 10..256 rows and checks WAVE channel count against selected graph track count.
2. FreshZero identity now checks the exact configured channel count and 1536-sample zero rows; its row check is dynamic.
3. The first selected Meter G0 blocker is Installed48 source-map construction. Its selected primary-binding validator still requires the one captured 38-channel job's exact projection/source/metadata hashes, file/data byte extents, sample count, and channels=38. This is narrower than a generic 38-row gate and will also reject another 38-row resource unless a matching selected binding is built.
4. After that map is generalized, NormalControlPreparedSnapshot remains fixed to channels=38, bed=10, objects=28, physical rows=38, and regular rows=37.
5. Installed48 source/control APIs still use fixed 38 physical rows and 37 regular rows. FrontendControlState canonical snapshots likewise validate fixed control38/regular37 extents.
6. A dynamic F01E cold-provider projection exists and derives physical/bed/object counts and zero-initialized object slots from the admitted graph. It resets callback index for the cold value but does not create the normal callback after-state or bind a DDP frame-view descriptor.
7. The selected Meter production callback still calls normal_control_prepared_snapshot and the legacy propose_normal_control_snapshot. The separate dynamic continuation proposal exists, but it is not connected to this selected callback path in the audited source.

The separate internal AtmosMezz job-configuration branch still checks 38, but the audited selected CLI request emits the ADM branch. Do not use that unrelated branch to classify the selected input path.

## Scope

This is a read-only current-source inspection, not a 36-row CLI run or DEE parity result. It identifies the current product gates so implementation can change them directly. Typed physical-row-to-source-ID mapping and playable 36-row output remain separate checks.
