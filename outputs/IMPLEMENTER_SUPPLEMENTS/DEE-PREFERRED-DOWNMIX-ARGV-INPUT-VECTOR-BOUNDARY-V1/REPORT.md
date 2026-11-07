# DEE command-line input-vector boundary

## Finding

The Ghidra package binds `DdpjocEncoderApp +0x318` to a Boost `vector<string>` option described as a repeatable input-file path. The runner copies that member into its argument structure, where the consumer at `param_2 + 5` iterates 32-byte strings and wraps each in a 40-byte input configuration record. That consumer does not split colon/equal fields.

Therefore `param_2 + 5` is the input-path vector, not the DME `--encoder` key/value list. This corrects the earlier interpretation of that vector.

## Open boundary

The dedicated `--encoder` option storage, delimiter parser, and argv-to-map owner remain `UNBOUND`. Actual child argv was not captured. A separately reused preferred-downmix typed reader does not establish that the composite reaches this input vector or the audio/BSI writer. Do not infer a DEE `--encoder` mapping or its audio effect from this package.

The upstream Wash manifest SHA-256 is `6b27f90541a950ac46e4f6e466de2444d58074bff19aa5621837e09e7b0da344`; normal and `python -O` checkers passed. No separate package seal is present. The target image hash comes from the parent project record and the installed image was not rehashed.
