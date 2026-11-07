# F01E dynamic continuation is not yet on the selected production path

## Source snapshot

This audit is bound to committed `HEAD` `31a21d72797d487fefe102581e5e83e1334b4d74` / tree `aa17672d0d70592e2548a93581e2c8d08ac528bb`, while separately hashing the current dirty worktree. Do not treat committed HEAD and work-in-progress as the same state.

## Finding

The selected CLI Meter production callback still calls `NormalControlPreparedSnapshot` and `propose_normal_control_snapshot`. `validate_selected_profile` still requires source and profile channel counts to equal 38. Therefore a 36-row input is rejected before the selected Spatial callback on this path.

The dirty worktree adds a dynamic F01E continuation proposal for 36/38 rows. It preserves prepared-input identity, programme/attempt/callback lineage, source resource and ADM metadata hashes, cursor/read indices, row count, owner generation, and source-origin generation. It is a borrowed noncommitting after-state and has no canonical digest, installable owner replacement, or GTC receipt authority. Its only caller is the new test.

The committed 36/38 geometry test directly exercises dynamic source geometry and outer-matrix production, but not `continue_meter_spatial_production_callback` or `SpatialControlMachine`. The new continuation test advances callback 0→1 and validates fresh-zero receipts for deliveries 2–4, but also does not call the selected production callback.

## Disposition

This is a concrete implementation gap, not an algorithm-proof demand: connect the already present dynamic continuation/geometry path to the selected Meter production callback and SpatialControl stage for the validated 36/38 input shapes, preserving existing identity and generation checks. Keep the arbitrary-N and native-scope limits explicit. Do not call the current tests end-to-end production coverage.

## Evidence scope

- Upstream package `F01E-WORKTREE-REAUDIT-WASH-V1`, manifest SHA-256 `9f7f63634d6dd6a5fc8a7482dd46f9170e71c2d3db510f7a4ad2b05d61feb109`.
- Wash recomputed all four package payload hashes and the current source-file hashes listed in `EVIDENCE.json`; the local Git ref still points to the listed HEAD commit.
- No tests were run and no source was changed by this audit.
- The package excludes raw implementation source; this supplement contains only the scoped findings and file-hash identities.
