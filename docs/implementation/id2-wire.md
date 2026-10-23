# F13D2 ID2 wire slice

This direct slice contains three stateless operations from the hash-bound F13D2 package:

- exact external DRC profile label to the selected ID2 U3 code;
- local extraction and strict validation of the Type11 header fields needed to obtain its LFE flag;
- serialization of the five explicit integer inputs into the selected 21-bit ID2 body, returned as three bytes with zero low padding.

The Type11 reader checks version 0, the short or extended count encoding, shortcut 1, and the count/LFE relation. It is a local prefix reader only; it does not validate or produce the complete Type11 body. The body serializer computes `phase_bit = phase_indicator != 0` and `lfe_bit = (format_mask & lfe_mask) != 0`, preserving integer bitwise-AND semantics.

Tests consume a seven-entry subset (`id2-profile-enums`, `id2-profile-from-header`, and `id2-field-wire`) from the approved unit-vector asset. They verify the subset against that asset's hash-pinned identity but pass only the selected cases to the implementation tests.

This code does not implement a live line/RF DRC producer, extend programme-level profile admission, produce Type11, enqueue ID2 into the FIFO, serialize the enclosing EMDF, own source/AU clocks, or commit programme state. Full F13 and product readiness remain deferred.
