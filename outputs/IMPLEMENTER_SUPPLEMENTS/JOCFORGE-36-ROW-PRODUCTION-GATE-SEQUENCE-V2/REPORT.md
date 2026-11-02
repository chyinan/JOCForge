# JOCForge 36-row production gate sequence

## Current first gate

The Rust RIFF reader, job binder, and Meter PCM cursor use actual channel counts, including 36 and 38. The static production path still blocks a 36-row input at Meter prepass callback 0 (G0): `build_meter_installed48_source_map` calls a selected-source validator fixed to 38 channels, 114-byte PCM24 block align, one selected sample count, and the 38/37/48 source-map shape. This is a static prediction conditional on real metadata/primary-control preparation succeeding; no 36-row CLI rejection was observed.

The current implementation HEAD advanced to `2f23885008cd789a5b17b639a1a3737b2ad87273` after the source audit. Wash independently found 50 of 51 listed source hashes still match; the only changed item is the architecture chain-status document. The listed production source files match the audited snapshot.

## Downstream work after G0

Opening the first map gate is not enough. The live Meter path still calls the legacy F01E normal snapshot fixed at 38/10/28/37, then builds 48/37 source geometry, Installed48 activity/renderer states and fixed source flags. The three Meter FreshZero drains have 38-row guards. MAIN roots still come from the legacy F01E provider; normal callbacks, Reader EOF carriers and four MAIN cached drains also require 38-row frames/controls. Finally, the Rust binder's dynamic physical count is not yet joined in the same job/generation to the native DDP process-frame channel count.

The smallest continuous implementation sequence is in `EVIDENCE.json`: dynamic source map with typed physical-row identity; real F01E dynamic owner/proposal wiring; consistent geometry/activity/Installed48 dimensions; same row identity through all Meter and MAIN drains; explicit absence semantics for global IDs46/47; then same-job/generation DDP frame-view count binding. For 36 physical rows with one direct LFE there are 35 non-LFE regular rows. IDs46/47 remain unbound and must not be silently zero-filled or aliased.

## Evidence boundary

The audit is source/static only. The authorized sample has metadata-only row-count evidence, not a real 36-row `PreparedJobInput`, Meter callback, Spatial callback, DDP frame view or complete EC-3/JOC output. No CLI, DEE, tests, PCM, media payload, key or license was touched.
