# Run29 DDP initialization and same-owner call chain

Disposition: NATIVE_INITIALIZATION_CHAIN_OBSERVED; INPUT READER AND SOURCE IDENTITY UNBOUND. This is a completed DEE reference invocation with bounded entry observations, not a current-product encode or a source-slot mapping.

## Provenance and verification

Source package spatial-selector-identity-wash-v13 manifest SHA-256: f30c5d04c35ec6087488d41599219c177d94ff08ea043c86e1d0d39329ba78d3.
NATIVE-DDP-INITIALIZATION-CAPTURE.json SHA-256: 01aac2c45cede7bd6e0baac3418a7b2e56369bffff8f74b1d9904bad6dac2cb3.
Parent v12 manifest SHA-256: 7b0bdaae6cf2825bf36e229cdca9372dbbe72da3c1c7cd50b033e12f841a60d0.
The source checker passed normal and optimized Python; Wash rechecked all manifest members. No audio samples or memory snapshots were read or retained.

## Run29 same-invocation observations

During one normal reference encode, the DDP process method and DDP state initializer entries were observed. Three ordered child entries followed on the same thread and same owner object, bound by thread stack and equal this argument within this invocation. The generic WAV/MediaInfo parser entry was not hit before normal process exit.

The reference output was 262144 bytes, SHA-256 077977564a8da534a52dee3feb1c791cc0825dd9cd7eb975575799b02450e686, matching the established reference artifact. Input fixture hash was 7826dcbef37ef69642a6164c97a37bcfb3f917cea74aa0f0a30d76d0a825f1d1. This is DEE reference output, not JOCForge product output.

Event order is limited to one process/thread. No ADM-sample clock, callback ordinal, or DDP generation counter was captured; do not join this order to run24/run27 or claim a generation.

## Still unbound

- ADM file reader/decoder that populates DDP frame input pointers;
- writer and capacity of the DDP frame pointer table;
- DDP slot to CHNA UID, physical track, or global source ID;
- selector source column to DDP frame slot;
- explicit DDP state generation identity.

The generic WAV/MediaInfo parser breakpoint remaining unhit does not prove which alternative reader ran. No model comparison, U04 numeric output, Gate1, full product output, or playback is claimed.
