# Wash review: R138 normalizer container type correction

This correction supersedes the prior description of RCX as a source-property map. Ghidra identifies RCX as a linked key/value record-list container. The normalizer looks up and rewrites entries; the internal input Run method appends the list to an input-run object field. It is not established as a typed source-metadata map or an audio-control map.

The static argument order remains RCX=record-list, RDX=timecode_frame_rate, R8=FFOA, R9=offset. The host list's connection to the DDP range consumer or setter slots 10/11 is UNBOUND. R141 did not capture normalizer values; the setter was NOT_HIT_FOR_THIS_RUN and no owner/generation join was captured.
