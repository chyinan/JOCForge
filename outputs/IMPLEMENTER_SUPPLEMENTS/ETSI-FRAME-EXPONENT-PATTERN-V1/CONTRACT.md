# Frame exponent strategy combinations

Public source: ETSI TS 102 366 V1.4.1 (2017-09), Table E.1.9, printed page 137 (PDF page index 136), and Table 6.4. Official PDF: https://www.etsi.org/deliver/etsi_ts/102300_102399/102366/01.04.01_60/ts_102366v010401p.pdf. The downloaded document is 1,833,684 bytes, SHA-256 `0229e151dfd9f8cec427f234798cac679a66fdec096feecc4d5ce455bbe3cadf`.

Table E.1.9 associates each five-bit code 0..31 with six ordered block strategies. Numeric two-bit meanings from Table 6.4 are R/reuse=0, D15=1, D25=2, D45=3. The following independently derived generation rule exactly reconstructs all 32 table rows in index order. It is an equivalent mathematical representation of the table, not a formula stated by ETSI in Table E.1.9.

1. Accept `code` as an exact integer in 0..31; do not treat a boolean as an integer. Start with a six-element U2 strategy row of zeros and segment start `a=0`.
2. For boundaries after blocks `i=0..4`, inspect bit `(4-i)` of the five-bit code. A set bit ends the current segment immediately after block `i`; its next segment starts at `i+1`. Append a final end at block 6. Thus the most significant code bit is the boundary after block 0, and the least significant bit is the boundary after block 4.
3. For each segment `[a,b)`, let `length=b-a`. Write at `row[a]`: D45/3 when length=1; D25/2 when length=2 or 3; D15/1 when length=4, 5 or 6. Every other block in the segment remains R/0. Continue with `a=b`.

This gives exactly 32 distinct `[6]` U2 rows; code is U5. For independent verification, concatenate the 32 generated rows in code order, each strategy as one raw U8 in block order. The 192-byte projection has SHA-256 `acfcaf98c12fa9688ef75943537a2f2f0a97a41ab75aa342468bd649b783d6bc`. This projection is an audit identity, not the on-wire packing.

For each of five FBW lanes, compare the complete six-strategy row with generated rows in increasing code order; the first exact match is that lane's frame code. This is the approved F09.4 lookup. Main and converter strategies are independently produced and matched; never copy main codes into converter codes. The LFE 1-bit strategy path is separate and does not use this five-bit table. F09 main uses per-block format if a lane has no table match. The washed F11 handoff identifies a converter frame-pattern mismatch as a producer failure; it must not manufacture a code or substitute an expected vector. Shape, noninteger/bool or values outside U2 are rejected by the existing selected input contract before lookup.

The generated rows were compared to the text extracted independently from the downloaded official PDF: all 32 code indices and all 192 strategy cells match, with 32 unique rows. This establishes the table reconstruction, not actual F11/F12 execution, output CRC, decode or audio acceptance. No research model, native DLL, PCM, EC3 or private key was used.
