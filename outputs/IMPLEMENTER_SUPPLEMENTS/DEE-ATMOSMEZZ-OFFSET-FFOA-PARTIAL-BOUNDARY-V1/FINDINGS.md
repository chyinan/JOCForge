# Offset and FFOA partial boundary

The two XML properties enter separate host slots (+0x168 and +0x148). For auto values, offset is selected from source metadata with a 0.0 fallback when absent; auto FFOA is rebased using source and selected offsets, then clamped not earlier than the selected offset. The first statically joined DDP consumer reads the normalized values separately; the FFOA path subtracts offset before its downstream helper.

These are static normalization/consumer rules only. The ADM/BWF chunk that supplies the named metadata, explicit numeric syntax/units, runtime owner/generation, and Reader/frame-alignment consumer remain UNBOUND. A bounded scan of six N36 and one N38 inputs found no discriminating named offset/FFOA fields or BEXT/iXML candidate. No DEE run occurred.
