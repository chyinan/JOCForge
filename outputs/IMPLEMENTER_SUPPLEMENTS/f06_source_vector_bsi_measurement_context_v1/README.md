# F06 source vector and BSI owner context

The source descriptor member +0x68 is a 24-byte begin/end/capacity header for opaque U64 qword elements. The per-source IO-set loop appends qwords from a context-selected source-map column; that vector is deep-copied to per-source record +0x30 at an eight-byte element stride. The separate record vector at +0x00 has 32-byte rows and is not the same vector.

The per-source record is forwarded to the F06 measurement consumer. A BSI serializer callback occurs inside the same measurement-consumer invocation; pointer-only captures report that the BSI context owner matches the serializer owner and differs from the source record. The pointer attempts lack process-bound module-load identity and success exit-code receipts.

The BSI serializer reads a signed U16 at owner +0x34. Its first writer, startup value and measurement update are not known; no qword element or U16 value was read. Do not infer numeric DialNorm lineage from a copied pointer or same call context.