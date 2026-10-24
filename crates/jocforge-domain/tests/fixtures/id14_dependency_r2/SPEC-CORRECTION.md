# Selected ID14 dependency correction — R2 evidence

This R2 package supersedes the R1 selected-consumption supplement and strengthens the F13C BSI correction. It confirms that the selected BSI count producer does not require encoded ID14 input. Two fresh private-owner calls used validated counts 1 and 16 with ID14 queue, EMDF serializer, and JOC body holder absent before and after; the constructor, producer, and importer returned 0 and produced/imported prefixes 0101 and 0110.

The base F13 spec Section 1 claim that this count must be derived by parsing serialized EMDF/ID14 is withdrawn. The verified selected source is the configuration count in the sibling `F13C-BSI-COUNT-PRODUCER` supplement.

The 392-event bounded header watch is supporting selected-path dataflow: 90 queue-copy context events, 270 output bit-writer read-modify-write events, 32 opaque serializer byte loads, 0 unclassified. It covers one source header and the first eight bytes of two actual queue buffers. It is not a global parser-absence proof. ID14 body generation, queueing and serialization remain required; this package only closes the encoded-ID14 dependency of the BSI count producer. Carrier-block source, broader F13C closure, and product acceptance remain open/not run.
