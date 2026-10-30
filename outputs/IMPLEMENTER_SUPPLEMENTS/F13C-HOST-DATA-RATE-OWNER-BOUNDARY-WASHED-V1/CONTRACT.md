# Host data-rate configuration to Atmos complexity boundary

## Observed configuration surfaces

Static host route metadata lists `atmos_mezz`, `atmos_mezz_to_atmos_ddp`, and `encode_to_atmos_ddp`. The host profile table contains `data_rate` options 448, 640, and 1024 plus an `auto` default. The inspected job template includes `atmos_mezz` and `data_rate=1024`, but the template was not runtime-validated.

The base DDP component has a `data_rate` setter that stores the value in its own core config object. The Atmos JOC component has a separate configuration descriptor and builds a caller-local workflow-parameter aggregate on measurement/encode paths. No typed owner/pointer join was observed from that aggregate or the host task field to the Atmos importer record fields that feed the selected BSI complexity byte. Equal field names do not establish object identity.

The separate `channel_configuration` route applies to `wav_list`; it is not joined to the selected `atmos_mezz` route. The bounded `dactlib.dll` scan found downmix configuration keys but no `data_rate` or complexity field. This is scoped evidence, not a global absence claim.

## Runtime and join status

The controlled native pair remains `NOT_RUN`. The prior `dee.exe --schema` attempt produced partial XSD because DDP plugins failed initialization and the process faulted; omissions from that XSD are not a rejection boundary. The host task field to Atmos plugin value, Atmos plugin value to importer record, and importer record to selected complexity byte remain `UNBOUND`.

No PCM, bitstream, key, native binary, or decompiled source is included. A valid accepted task route and same-run owner/config/BSI observation are still needed to close the join.
