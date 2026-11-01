# F06 BSI owner U16 conditional writer scope

This supplement isolates the confirmed setter facts for the signed U16 at BSI-owner-relative +0x34. The serializer consumes it as a raw BSI code, using special-tag cases or a five-bit path. A physical dB interpretation is not established.

Two conditional pre-serialize paths copy a fixed 36-byte block to the same typed BSI owner. In both confirmed blocks, the first U16 written at +0x34 is zero. The measurement update also processes per-source rows separately; those row values do not form this U16 assignment. The source qword vector is not read by either confirmed assignment path.

These are conditional static setters. The BSI owner constructor-time first write/default, other writers, the active guard in a native run, and the serializer-time U16 value remain unbound or uncaptured. The result must not be generalized to all modes or treated as a decoded dB value.
