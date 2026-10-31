# ADM object to ChannelInputId input boundary

## Washed result

For the hash-identified approved converter, an AO_XXXX object suffix is parsed as hexadecimal; values at or below 0x1000 are rejected, and the static conversion produces a typed ChannelInputId equal to suffix minus 0x1001. The converter then checks that ChannelInputId against IAB frame channel/source metadata.

The authorized ADM BWF metadata-only observation reports AO_1015 with one referenced audioTrackUID and CHNA track index 21 / physical host slot 20. The static converter result for AO_1015 is ChannelInputId(0x14). The equal integer values do not establish identity: ChannelInputId and the physical host slot are distinct typed domains.

## Native F06OF boundary

The selected PcmMd builder sets its OAMDI event-record count to zero and event-record pointer to null before the adapter call. Therefore this selected builder supplies no 48-byte OAMDI event record and consumes no corresponding U16 event key. This does not exclude other producers or modes.

The native descriptor U8 candidate, descriptor U16 OAR index, 48-byte OAMDI U16 event key, 96-byte OAR U16 key, ADM object/UID, ChannelInputId, SelectedGlobalTargetId, and TargetSlotIndex remain distinct domains. No native bridge to the F06OF target identity or generation is established.

## Runtime limit

DEE setup attempts failed before workflow execution because the current environment could not initialize the required DDP filter. Input processing is NOT_CONFIRMED and native capture is NOT_RUN. This is not a negative/no-hit result. No license or key diagnostics were read. No PCM or encoded payload is included.
