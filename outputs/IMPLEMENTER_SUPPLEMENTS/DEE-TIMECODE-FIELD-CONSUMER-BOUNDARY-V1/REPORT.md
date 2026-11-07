# job_config timecode_frame_rate boundary

Reference-only Wash supplement; it does not enable non-default frame-rate behavior. `IMPLEMENTATION_READY=NO` for the unresolved XML/runtime joins.

The templates list `not_indicated`, `23.976`, `24`, `25`, `29.97`, `30`, `48`, `50`, `59.94`, and `60`; they explicitly write `not_indicated`. Templates do not prove omitted-field runtime defaults.

On the host, `FUN_1400f13e0` matches the exact `timecode_frame_rate` key, gets a typed value via `FUN_1400705c0`, and stores it at object `+0x31`. The function body does not identify whether that owner is the `adm` input node, `atmos_mezz` input node, or both, nor a subsequent timecode consumer.

On the DDP plugin side, `FUN_18014fd90` reads a numeric `timecode_frame_rate` property from a map at `param_1+0xa0` and uses it in range/timecode normalization. The XML `/filter/audio/encode_to_atmos_ddp` value is not joined by owner/generation to that property map. The input-side and output-side XML fields are not proven to reach distinct runtime consumers.