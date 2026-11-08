# E-AC-3 chanmap U16 serialization

## Approved scope

This supplement covers only emission of the 16-bit `chanmap` value in an E-AC-3 dependent-frame header. For a 16-bit logical field emitted MSB-first, LSB-numbered numeric bit `n` occupies field slot `15 - n` (slot 0 is the first bit emitted).

For the approved 7.1 D0 target `chanmap=0x1A00`, set bits `[12, 11, 9]` therefore occupy emitted slots `[3, 4, 6]`; the field value is `0001101000000000`. The profile contract separately identifies its MSB-indexed set positions as `[3, 4, 6]` and lists the target coded locations. This supplement only checks numeric serialization and does not derive those semantic labels.

The absolute bit cursor, final file byte order, and D0 source-role mapping are outside this contract. In particular, do not interpret the three set bits as an ADM-row, coded-channel, or JOC-carrier count. `production_allowed=false` remains in force for the profile as a whole.

## Delivery boundary

This sanitized package contains only the serializer rule, input-only vectors, and hashes of the two Wash-reviewed inputs. It contains no Ghidra traces, source paths, binaries, audio, PCM, encoded payload, keys, or licenses. The 16 one-hot cases and `0x1A00` composite are input-only arithmetic vectors; execution against a native writer or generated bitstream is `NOT_RUN`.
