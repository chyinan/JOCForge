# ID14 row10 cross-sample consistency

Two hash-bound authorized inputs, Human Cat and My Jealousy, were observed at row index 10. Their packet row counts differ (38 and 36), but both observed raw scalars were byte +3 = 0xBA and U16 +4 = 0; the normalized consumer word was 0x800A and the bit15 test was taken. This is a two-sample consistency observation, not a semantic identity or a generalized transform rule.

The input hashes and shared job hash are bound in EVIDENCE.json. The My Jealousy run reached the U6 writer entry but stopped before the writer instruction; bitstream remains NOT_RUN. The row semantic name and typed ADM/EMDF producer remain UNBOUND.
