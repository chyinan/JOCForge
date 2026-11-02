# Current 36-row CLI gates after the dynamic Meter cursor change

## Wash disposition

The upstream source audit is bound to HEAD `2155ea31d6c78b5f91b7ba256ce0a506581d1618`. It corrects the previous blocker order: the Meter IO cursor's row shape now accepts 36/38 rows, but the first current CLI failure is the Meter G0 Installed48 source-map admission, before F01E. Fixed F01E and FreshZero guards remain the next gates.

## Current ordered path

1. **Meter IO cursor — shape gate cleared statically.** It accepts channel counts 36/38, checks PCM24 block alignment as `channels × 3`, stores the actual row count, and allocates normal/FreshZero rows from that count. The PreparedJobInput Meter view checks the graph physical-row count.
2. **Meter G0 Installed48 map — first remaining failure.** CLI Meter prepass runs before MainSpatialColdRoots. `stage_independent_meter_g0_normal` first builds a source map whose selected binding still requires 38 channels and block_align 114; 36-row PCM24 has block_align 108 and yields `Installed48SourceMapError::SourceExtent`, wrapped as `IndependentMeterG0Error::SourceMap`. If that gate is widened, a second `controls.row_count()==38` check remains.
3. **F01E Meter normal snapshot — following gate.** Meter G0 constructs a machine from fixed `providers.f01e` and calls the legacy `normal_control_prepared_snapshot`, which requires 38 channels/28 objects and rejects 36/26. The dynamic PreparedJobInput owner/proposal API is not used here.
4. **Meter FreshZero application validator — later gate.** `validate_fresh_zero_callback_identity` still requires exactly 38 physical rows.
5. **Main Spatial F01E — not reached for 36.** The spatial root also starts from the fixed provider and stages the legacy F01E normal snapshot.

## Dimension boundaries

The R4 native row-map receipt supports dynamic 36/38 config-map row counts under its static selected-child conditions; it does not establish a zero-padding rule. The separate F03W global source axis remains 48 wide. Under the approved 36/10/26 model, 35 regular rows map to source IDs 11–45; IDs 46/47 are unassigned, and native map rows 36/37 are not joined to them. Do not equate input rows with fixed output/codec slots, and do not synthesize audio rows for the two unassigned IDs.

The source graph shows that after these gates the existing spatial/F07-F15 candidate path, source-advance transaction, durable AU append, and F15 finalize/expose can be reused. This source audit did not run a 36-row encode, DSP, tests, decoder, or playback; playable output remains unproven.
