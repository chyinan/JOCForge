# Current 36-row Meter gate order — superseding source-path correction

This V2 supersedes the specific Meter G0 gate attribution in JOCFORGE-36-ROW-CURRENT-PRODUCTION-GATE-ERRATUM-V1. The previous version confused two different Installed48 builders.

## Current selected Meter G0 call path

The selected Meter normal stage calls build_meter_installed48_source_map, which calls build_selected_installed48_source_map and validate_selected_source_binding. That validator is dynamic: it accepts ADM channel counts 10..256, requires 48 kHz PCM24, checks data_bytes is divisible by channels*3, and matches the sample count to the source extent. The builder derives physical_host_count from metadata row count, regular_count as N-1, and active_source_count as 11+regular_count.

After this dynamic source-map call, the selected Meter stage calls PreparedJobInput::normal_control_prepared_snapshot and the legacy propose_normal_control_snapshot path. NormalControlPreparedSnapshot::validate_selected_profile still requires channels=38, beds=10 and objects=28. Its prepared snapshot and the downstream Installed48/SpatialControl state still use physical/control rows 38 and regular rows 37. This is the first fixed-cardinality gate on the inspected selected Meter G0 route.

FreshZero callback identity checks are dynamic in the current source: it compares row count to configured N and validates 1536-sample zero rows. Do not describe FreshZero itself as a fixed-38 gate.

## Separate 38-channel primary-map path

build_primary_installed48_source_map is a different helper that calls validate_primary_binding and contains exact selected-38 resource/hash/sample checks. Do not attribute that predicate to build_meter_installed48_source_map. Whether every full MAIN path reaches this separate helper is outside this specific call-path erratum.

The dynamic source map assigns regular source indices in physical-host iteration order in the current JOCForge model. That is a code fact, not proof that DEE uses the same row-to-OAMDI mapping; keep that identity/equivalence question separate.

## Status

The 36/38 target shapes and new-attempt reset contract in the prior row-shape package remain unchanged. This correction changes only which current source gate is first. No 36-row CLI encode, DEE comparison, or PCM parity was run.
