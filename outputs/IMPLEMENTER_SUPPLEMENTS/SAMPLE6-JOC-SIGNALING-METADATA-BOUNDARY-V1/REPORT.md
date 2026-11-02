# Sample 6 JOC signaling metadata boundary

FFprobe recognizes the verified current output stream as E-AC-3 / Dolby
Digital Plus + Dolby Atmos at 48 kHz, 5.1(side), 1024 kb/s. It reports 4,247
packets and 4,247 frame metadata entries of 1,536 samples each. The only
frame side-data exposed is `Metadata relevant to a downmix procedure`.

This FFprobe metadata surface exposes no explicit per-AU JOC, Atmos, or EMDF
flag. The stream-level Atmos profile is positive stream recognition, but is
not a per-AU coverage count. Downmix side-data is not JOC/Atmos signaling. Do
not infer that all 4,247 AUs carry JOC metadata, and do not infer the opposite.

This is metadata-only parser evidence; it does not test spatial render or
listening quality. No raw packet, encoded payload, or PCM was retained.
