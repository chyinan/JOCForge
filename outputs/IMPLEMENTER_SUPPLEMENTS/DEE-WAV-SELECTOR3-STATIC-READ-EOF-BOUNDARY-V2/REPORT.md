# WAV selector-3 static reader and EOF boundary

## Bound static path

The hash-bound Ghidra 12.2 package binds the WAV suffix to selector 3. FUN_1803ecb80 registers open callback FUN_1803ec660 in registry slot param1+0x20. The callback constructs type 3 through FUN_1804d14c0 and FUN_1804cfc80; the constructor installs atmos_storage_master_s::vftable and stores type byte 0x2f. On open success, the object is returned to the caller.

The vtable +0xd8 entry resolves to FUN_1804ccb00. In normal access mode the body reads lower-storage current and limit through slots +0x60 and +0x68. It proceeds only when current < limit and requested_count != 0, caps the request to limit-current, then caps the returned count to the lower read's actual count. When current >= limit the returned count stays zero.

## Range and runtime boundary

The parent range receipt represents a segment as startSample plus length=(endSample-startSample), accepts endSample==source_sample_count, and uses start+length-1 only as an arithmetic last-index candidate. For the bound full-file sample_count 6,522,906, the candidate index is 6,522,905.

The static read body is now bound. The historical receipt still has no runtime provider pointer, selected branch, actual read request, or returned count. Therefore actual inclusion of the final sample remains NOT_CAPTURED; this package does not claim an observed runtime EOF result.

This V2 supersedes the prior Wash supplement's static callback-target UNBOUND boundary. It does not remove the runtime-selection or final-sample limits. No new DEE run, decode, PCM/payload/key/license access, or implementation-repository read occurred.

