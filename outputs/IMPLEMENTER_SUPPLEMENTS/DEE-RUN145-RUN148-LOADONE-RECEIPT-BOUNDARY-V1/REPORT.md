# Run145/Run148 historical LoadOne receipt boundary

## Scope

This addendum audits which existing Run145/Run148 artifacts actually bind the ordered plugin-load list and per-module audio_filter getter counts needed for an absolute DDP Atmos manager/Runner ordinal. It also records the static license-precheck gate before StageRunner construction. It does not infer historical load order from installed files or from unbound loose logs.

## Sealed receipt findings

- Run145's hash-bound RESULTS/preflight/completion artifacts bind the selected DDP Atmos image/path but do not contain an ordered LoadOne module list or per-module getter counts.
- Run148's hash-bound capture preflight and selected-route receipt bind input/job and DDP image/process entry, while explicitly leaving the complete audio_filter set unbound/not captured.
- The R4 loose Run145 and Run148 logs were hashed by the incoming audit, and no loader-order/count markers were found in them. Their hashes are not referenced by the original run preflights/result manifest, so their missing markers are not treated as authoritative proof of historical non-occurrence.

## Consequence and minimum evidence

The conditional ordinal formula remains `sum(getter counts of audio_filter modules preceding DDP Atmos in the run's effective LoadOne order) + DDP Atmos local slot`. Run145 and Run148 absolute ordinals remain `UNBOUND`. The minimum historical evidence is an ordered, same-run LoadOne receipt for each run, with module basename, image identity, and audio_filter getter count per entry; identify DDP Atmos by image SHA-256 `3d66bcec36031fd48e6565d15f05fea656642377ca4f8c98cdce1cce8b7e95d2`.

The static host path includes a license-content parser prerequisite before StageRunner/RegisterPlugins/LoadAll. No host was started and no license contents were read. A no-job runtime probe was not executed. No PCM, media payload, key, or bitstream was read.
