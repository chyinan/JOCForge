# Atmos DDP 5.1 trim metadata boundary

Static declarations expose string choices: `surround_trim_5_1` uses `auto`, `0`, `-3`, `-6`, `-9`; `height_trim_5_1` uses `auto`, `-3`, `-6`, `-9`, `-12`. Both typed descriptors use `auto` as their declared initial value. The descriptor does not itself specify a numerical unit or precision. Exact host omission behavior and invalid-token rejection were not observed.

The inspected path binds the fields to typed metadata; it does not establish any audio, signaling, or output effect. A separate binary string mentions a CBI restriction but its selected runtime path was not established. A/B vectors are inputs only and were NOT_RUN in this evidence slice. No native code, decompilation, media, PCM, payload or private key is included.
