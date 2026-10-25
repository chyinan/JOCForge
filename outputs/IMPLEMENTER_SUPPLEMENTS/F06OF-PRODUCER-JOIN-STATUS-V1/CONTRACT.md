# F06OF producer join status

Status for the selected ADM BWF → 38-channel, 48 kHz meter path:

- `CONVERTER_CHANNEL_INPUT_ID_TO_PCM_PHYSICAL_ROW = UNBOUND`
- `PCM_PHYSICAL_ROW_TO_SELECTED_METER_DESCRIPTOR = UNBOUND`
- `NATIVE_CURSOR_AND_EOF_FOR_SAME_SOURCE_GENERATION = UNBOUND`
- `F06OF_MODEL_CLOCK = DEFINED_BY_APPROVED_MODEL_SPEC_ONLY`
- `IMPLEMENTATION_READY_FOR_PRODUCER_JOIN = NO`

The previously implemented converter-local map ends at `ConverterChannelInputId`. That map does not prove which decoded channel writes a physical PCM row, how those rows are ordered for the selected measurement descriptor, or that the descriptor and reader belong to the same source generation.

Static evidence partially closes only the downstream consumer: the selected meter callback checks the configured channel count and accepted frame-size/state conditions before planar PCM and metadata processing. The descriptor's producer, metadata-row semantic owner, and identity link to the converter storage remain unresolved. These consumer observations do not define a replacement producer algorithm.

The approved F06OF model contract continues to define the model callback schedule and cold/reset behavior. Those rules are not native cursor, EOF, or drain observations. Existing callback counters without a same-resource and same-generation receipt must not be used to claim this join.

## Minimum evidence needed to close the join

One hash-bound receipt must bind, for the same selected resource and generation:

1. opaque prepared-input identity;
2. selected semantic channel ID, CHNA source-channel index, track index, and physical row ordinal;
3. descriptor owner token/generation and the row it consumes;
4. reader requested and returned sample counts, cursor before/after, and explicit EOF status;
5. callback/source index for normal input and each drain, with no raw PCM or row body.

No dynamic observation is authorized by this status package. Any capture that is needed to establish this edge requires a separate Wash GO naming the exact job, guard, bounded receipt, and stop condition.
