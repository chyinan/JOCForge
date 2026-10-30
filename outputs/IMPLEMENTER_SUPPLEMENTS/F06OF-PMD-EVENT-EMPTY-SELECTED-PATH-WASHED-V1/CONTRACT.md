# Selected PcmMd OAMDI event path boundary

## Static native fact

In the selected PcmMd builder examined by the hash-bound v2 addendum, the OAMDI event count is set to zero and the event-record pointer is null before the OAMDI setter call. Therefore this selected builder supplies no 48-byte OAMDI event record and no U16 event key to that setter.

The same PcmMd has a separate descriptor wrapper with a count and pointer to six-byte entries. The OAR path derives separate candidate indices from descriptor bytes and a separate 96-byte OAR event key. These domains are distinct from the 48-byte OAMDI event U16 key and from F06OF target slots.

## Limits

This is a static, selected-builder boundary. It does not prove that other PcmMd producers or modes cannot populate OAMDI events. Native capture and fresh model replay remain `NOT_RUN`. No ADM identity, TargetSlotIndex, control16 word, or generation join was established; F06OF crosswalk remains `UNBOUND`.
