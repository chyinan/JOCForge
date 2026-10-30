# DEE host stream-object path to DDP; ADM identity remains open

Disposition: PARTIAL_STATIC_HOST_PATH_EVIDENCE. This reconciles DDP adapter v8 with v7 frame-channel mapping and the earlier selector reference. It does not identify an ADM channel ID or close U04/Gate1.

## Source binding

v8 source package manifest SHA-256: 6b2aeb764ddea372e7f3bd43a09a96597716d64a24463e9ae36e1e59ad3866f8.
v8 EVIDENCE.json SHA-256: ad588338874bbfe011cfed96134996fa56c04960b51307ad03f12092a9097f51.
Superseded v7 manifest SHA-256: 1ff077f7e348212df989762d34928da28546473ec022530afcf2dd5072d1c27e.
Prior selector reference manifest SHA-256: 55ec6020b9b5f5f5802a359febaf37498592621b8b026e9a407d67e0d121892f.
The v8 checker passed normal and optimized Python; Wash rechecked every listed source member hash. The native/static trace was not independently reproduced by Wash.

## Host stream-object path reported by v8

The AtmosMezz input Run method obtains a stream object from its configured storage provider and stores it under stream_object in the input-stage property map. DEE RunInput resolves the configured source target's stream_object and stores that object in the StageRunner input-node record. RunAudioFilter iterates the input node and passes the stored object to the audio-filter process method. The source package marks the static property-write/read and StageRunner transfer as bound.

This closes a host object-transfer path to the DDP process argument. The storage-provider method and the writer of the object's PCM pointer table/channel-config rows are not joined to ADM AudioTrackUID or CHNA track index.

## DDP frame index edge

Combining v7 with v8, the static non-mode-1 path reports that local DDP slot i maps to frame input channel slot i: one u16 channel-config count governs both six-byte config-row copying and PCM channel copying, and input pointer-table slot i is copied to internal buffer slot i. The runtime mode branch and values for a reference call were not captured. The top-level 96-byte per-source count remains a separate count domain and is not proven equal to selector width 48.

## Separate selector reference identity

The prior completed selector reference invocation maps columns 11..47 to regular global source IDs 11..47 using the approved F03R crosswalk. It used the same authorized 38-channel BWF hash 7826dcbef37ef69642a6164c97a37bcfb3f917cea74aa0f0a30d76d0a825f1d1, but a different job XML from the current accepted job. It is bounded same-input reference evidence, not the current job run.

No typed receipt or same-generation pointer identity joins the DDP frame channel list to those selector columns, the CHNA UID table, or the F03R physical-slot crosswalk. Keep all those edges UNBOUND until directly joined.

## Retry and remaining work

Run14 added the required misc/temp_dir to the legacy XML and used the complete D-drive test installation. The DDP family still failed initialization, the workflow reported Unknown filter, no DDP frame hook ran, and no reference output was produced. This failure is not a native measurement. Gate1, model replay, U04 numeric output, Type10, and six-plane validation remain NOT_RUN.

Next trace the storage provider's stream-object construction and PCM pointer-table writer back to the ADM BWF reader/CHNA UID and physical-slot receipt. Then bind one explicit DDP frame input channel to the existing selector source column and approved F03R crosswalk. Do not join by ordinal/count/shape. No source code, decompilation, process address, raw payload, license, or key is copied.
