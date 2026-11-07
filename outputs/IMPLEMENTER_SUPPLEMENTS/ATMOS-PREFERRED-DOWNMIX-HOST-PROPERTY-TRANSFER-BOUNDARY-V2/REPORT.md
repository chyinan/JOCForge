# Preferred downmix host property transfer, method-bound

## Static app-source path

The current hash-matched DME `app.asar` contains a concrete `getEncoderConfig` dispatch from the Dolby Digital Plus Online Media encoder to `getDolbyDigitalPlusOnlineMediaConfig`. Within that exact method, the `PreferredStereoDownmix.active` value and `handlePreferredStereoDownmix(value)` result form the `preferred_downmix_mode` entry in `valuesToCombineMap`.

The same method iterates that map, gates on `propertyIsActive(value[0])`, appends `key=value` to a composite command string, and emits it as `PropertyKeyValue("--encoder", cmd)`. The preferred key is preserved inside the composite `--encoder` argument; this is not a standalone `--preferred_downmix_mode` property and the excerpt does not rename it to `dmixmod`. This binds the exact static app-method path, not a runtime UI event or child-process invocation.

## Native boundary remains open

The schema-validated `atmos_mezz` LoRo/LtRt job pair is a separate prepared artifact, not a runtime DME job. No same-run app property/inputType event is joined to the candidate XML or selected `encode_to_atmos_ddp` provider.

Native DDPJoc receipts identify a separate `dmixmod` key whose first proven text consumer performs presence/allowlist validation. No typed native owner/setter or mapping from composite `preferred_downmix_mode` to `dmixmod` is established. Thus the host app serialization path is now bound statically, while host argument → selected native configuration setter and the following typed `ddp_dmixmod` consumer remain `UNBOUND`. Audio/BSI/UI-only/ignore classification remains `UNBOUND`; spatial rendering is `NOT_COVERED`; candidate A/B is `NOT_RUN`.

## Provenance

Upstream manifest SHA-256: `10e1580e55c9b33f4b65c17fa00b8060eab81ad71227c210cc491e8638dc6979`; SHA256SUMS file SHA-256: `2248f9f96bf49e4f88ddad0cd9e0bda0c32da832d16e50e7cb5d58c180694a8f`. The upstream normal and `python -O` checker passed, verifying five payloads, six checksums, and eight parent receipts. The current `app.asar` hash was recomputed by the source pass and matched its prior source receipt. No raw app source, media, PCM, payload, key/license, or reverse script is included here.
