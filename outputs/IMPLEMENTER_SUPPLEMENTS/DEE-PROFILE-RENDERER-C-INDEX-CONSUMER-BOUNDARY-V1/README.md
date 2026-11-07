# Profile/renderer C-index consumer boundary

The selected descriptor-bearing ADM path passes a separate count `C=16` to profile/renderer metadata initialization. `FUN_1801ad360` consumes C 0x34-byte input records, packs their fields into 30-byte profile records, and `FUN_1801261d0` stores them in an indexed profile-object table. This is a distinct count domain from source rows `N` and canonical target rows `M`.

The input record array and destination profile table have different owners. The bounded static slice did not identify the input array's first value writer, so it does not prove how those C records originate. `FUN_1801261d0` itself neither reads PCM samples nor writes EC-3/BSI; a later audio/BSI use is not established, and no global “profile-only” claim is made.
