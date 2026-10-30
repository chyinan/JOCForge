# Run-100 body-length to F13 container identity boundary

## Observed native length chain

Run-100 recorded 4,247 same-parent count/codec events. In each event, the counted body-byte value matched the codec-record, planned, serialized, refill, and U16 body-length-prefix values. The observed range was 232–375 bytes. Block-2 reservation fields and the 9-bit writer operand matched counted body bytes plus 4; the writer advanced 9 bits.

## Static corroboration and limit

Ghidra analysis of the hash-identified Atmos module completed targeted analysis for the selected parent, count, and serializer functions. The parent contains a scalar anchor and calls both the codec and serializer; reservation and refill paths call the writer. The whole-program analysis timed out at 180 seconds.

The run-100 event stream records generic container/serializer/refill/reservation lengths, owner aliases, and return status. It contains no EMDF type or payload-field tag for the counted object. The inspected parent/count/serializer static relationship therefore does not bind the value to a typed F13 EMDF container or the same source-generation owner.

## Remaining scope

Counted body bytes to complete F13 EMDF container identity remains UNBOUND. Parent invocation to ADM sample/frame/source generation remains UNBOUND. Writer operands 507/508/511/512 and 512 overflow behavior remain NOT_OBSERVED. The selected model producer upper bound is conditional on that still-missing identity; it is not a native global-unreachability proof. No audio, PCM, EMDF/EC3 payload, private key, binary, or decompiled source is included.
