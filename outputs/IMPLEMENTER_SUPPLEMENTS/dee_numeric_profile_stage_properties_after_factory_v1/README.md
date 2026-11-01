# Numeric profile fields and factory-index boundary

For hash-identified Run145/Run148, the route name encode_to_atmos_ddp selects the callback-created config by exact name and length. Numeric properties such as data_rate=1024, speech_threshold=100 and LORO mix levels are applied to the already-created stage through SetStageProperties. preferred_downmix_mode=loro is a string property, not a numeric loader index.

The audio-filter loader index comes from the callback-array loop counter; no typed transfer from the inspected numeric task fields to that index was found in this route. Decoded output metadata profile_index=3 is a separate domain and is not an input selector. Global callback ordinal remains order-dependent.
