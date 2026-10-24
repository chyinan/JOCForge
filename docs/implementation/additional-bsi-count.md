# F13C selected additional-BSI count

This direct slice derives the selected count from an admitted rate code and signed-I16 channel labels. It also models the observed configuration-count status/slot result, writes the two supplied payload bytes, and copies the complete 64-byte payload into a typed importer projection.

The module does not define a serialized ID14 parser or derive count from EMDF. A separate R2 observation confirms that selected BSI calls with validated counts 1 and 16 succeed while the ID14 queue, EMDF serializer, and JOC body holder are absent. This does not prove those components are unnecessary elsewhere.

The configuration constructor’s header/selector prefix bytes are not available. The Rust outcome reports the observed “prefix changed” fact and exact count-slot/status result; it does not synthesize prefix bytes. Invalid native inputs map to status 220 while preserving the tested count slot. No whole-object rollback is claimed.

Bypass validation probes, other substream counts, the earliest 64-byte tail initializer, carrier-block identity, and ID14 body parsing/serialization remain outside this slice. F13C and F13 remain deferred.
