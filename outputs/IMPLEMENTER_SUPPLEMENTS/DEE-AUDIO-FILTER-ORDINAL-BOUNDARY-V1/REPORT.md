# DEE audio-filter ordinal boundary

## Scope

This supplement binds the selected DDP Atmos module's local callback slot to the host manager-vector and Runner configuration-list ordering. It does not establish the selected callback's absolute global ordinal for any particular run.

## Bound facts

- The selected DDP Atmos image has SHA-256 `3d66bcec36031fd48e6565d15f05fea656642377ca4f8c98cdce1cce8b7e95d2`. Its local callback table has three entries at RVAs `0x12a440`, `0x12a4c0`, and `0x12a540`; the selected route is local slot 0 through the previously washed selected-task binding.
- For a hash-matched image, the loader reads local entries in ascending index order and appends them to the host audio-filter manager vector. The module-path list is walked in its stored order; the traced path contains no sort/reorder step.
- The Runner enumerates that manager vector from index 0 and appends one config object for each callback result. The callback ordinal is therefore preserved as the corresponding Runner config-list ordinal when the complete list is present. The eligibility-disabled path still appends its config object.
- Conditional formula: `global_ordinal = sum(callback_count of modules preceding DDP Atmos in this run's loaded module-path order) + local_factory_index`.
- The DDP Atmos local slot order is the same for the same hashed DLL image in the two existing run summaries. This does not establish a stable global ordinal across runs.

## Scope limits

The complete ordered audio-filter module set and the preceding-callback-count prefix for either run are absent. Thus the absolute global ordinal, cross-run global-ordinal stability, and absolute Runner list index remain unbound. The direct factory callback entry was not observed; descriptor owner and generation remain unbound. No new DEE or WinDbg run, no PCM/payload read, and no key/license access are included. This evidence does not establish an audio algorithm rule or whole-chain readiness.

## Evidence provenance

The source evidence package manifest SHA-256 is `8c47327c3b2d37e730cab7066b001bb2aa3d00e68a4b314c15a004b29e706c85`; its detached seal SHA-256 is `f658f9a432ba0424d8aa9d8eee098f0e1b4d4de2ca6fe6e201138b1781d81dbe`. The Wash review independently ran that package's checker normally and under Python optimization; both returned `VALID_PARTIAL` and verified all 27 indexed source-file hashes.

The selected callback/stage identity is bound to parent Wash package manifest SHA-256 `00856107e8c19c232e7a560dd723e3cce804d6d2da8003144b43bf1f5484f25c`. The implementation-facing claims here are restated as typed ordering relations; no decompilation or source text is included.
