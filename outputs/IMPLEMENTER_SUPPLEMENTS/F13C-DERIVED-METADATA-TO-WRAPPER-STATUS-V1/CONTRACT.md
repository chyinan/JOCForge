# Derived descriptor metadata to profile wrapper

Status: `D_METADATA_TRANSFORMS_STATIC_CLOSED; F12_CONFIGURATION_COUNT_IDENTITY_UNBOUND`.

On the reviewed encoder construction path, a worker descriptor supplies three metadata domains to an open configuration that is then passed to the profile-input wrapper. This is conditional static behavior and does not identify the selected measured source/job.

## Channel/map metadata

The descriptor's U16 channel count is zero-extended into a U32 configuration field. On the branch whose mode is not 1, the adapter copies `channel_count` records of 6 bytes into scratch storage and places the scratch reference in the open configuration. On mode 1, the map copy is omitted and channel counts 10, 12, and 16 map to codes 2, 1, and 0 respectively. No selected mode value is captured.

## Auxiliary metadata and rows

If the auxiliary metadata reference is non-null and the mode is 0, the adapter copies 120 bytes (seven 16-byte chunks plus an 8-byte tail) to scratch storage and passes its reference to the open configuration. The open-context constructor copies the record into a separate context-owned area and runs an in-place scalar range normalizer. A separate optional-settings block is copied as 44 bytes to another context-owned area. Contents and actual scalar values were not read.

The metadata-row vector is required to contain exactly 9 records. Each 28-byte record is transformed by skipping its first 4 bytes and copying the remaining 24 bytes to a context-owned 216-byte array. No meaning is assigned here to the skipped field or to any row body.

The same local open-configuration object is passed to the profile-input wrapper, which copies its fixed 0x78-byte prefix to wrapper-owned storage. Its signed profile-pair key 13 remains the separately documented `X + Y` expression; that field is not identified with the F12 configuration-record parameter 13.

## Open identity edge

The descriptor-derived configuration fields and wrapper prefix are not joined to the separately validated F12 configuration-count producer input or to a common source/reset generation. Keep the D-derived metadata and F12 count as distinct owners. Do not infer the missing join from field names, equal values, channel/row counts, or shared module identity.

Selected branch values, source identity, and F12 producer ownership remain `UNBOUND`. No additional F12 algorithm rule or product-ready claim follows from this static transform map.
