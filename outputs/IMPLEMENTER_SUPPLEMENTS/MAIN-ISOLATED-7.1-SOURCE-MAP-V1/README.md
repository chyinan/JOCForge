# JOCForge main ↔ isolated source map

Status: `WASH_SOURCE_MAP_REFRESH`; architecture reference only; `production_allowed=false`.

This package refreshes the existing main-branch reuse matrix against the current `JOCForge-isolated` source snapshot. It maps the legacy main implementation to current isolated modules, identifies the useful design pattern, and marks the current disconnected seams that still block a production I0+D0 route.

## Snapshot

- Main source: `D:\Programs\JOCForge`, ref `main`, commit `05a4108e0c6288130dec1203b301979a91475fca`. The checked-out main worktree is the migration scaffold, so main source observations were read from the named Git ref.
- Isolated source: `D:\Programs\JOCForge-isolated`, branch `experimental/independent-encoder`, HEAD `1acf9c2610edb7da54383481784bf0278e28a0cf`.
- The isolated worktree has existing documentation and scratch changes. The mapped Rust source paths had no working-tree diff; no changes were made.
- The earlier `FLAT7X-MAIN-ARCHITECTURE-REUSE-MATRIX-V1` is sealed to isolated HEAD `9456d4ca2c105ba2c799bf24a1545af2eb3e1d48`. This refresh supersedes its isolated source map while retaining its main commit and target-rate facts.

## Contents

- `SOURCE-MAP.md`: detailed module/function mapping and the shortest maintainable integration path.
- `REUSE-MATRIX.json`: machine-readable reuse decisions.
- `SOURCE-RECEIPTS.json`: repository identities, file hashes, and source locations.
- `build_source_map_package.py`: rebuilds the manifest, seal, and ZIP.
- `verify_package.py`: verifies package inventory, hashes, target contract, and seal.

No source code, audio, PCM, EC-3 payload, or proprietary encoder material is included. No codec execution or tests were run for this source review.
