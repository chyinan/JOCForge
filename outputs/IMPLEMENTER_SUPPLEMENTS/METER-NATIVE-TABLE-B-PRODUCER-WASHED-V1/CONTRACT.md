# Native table-B producer and consumer lineage

## Bounded native finding

For the selected native route, the task-config ordered map is cloned into the selected SpatialCoder configuration. `SpatialCodingState` constructs two 52-byte-stride tables. The table-B writer iterates map entries by local ordinal, reads one U32 value, normalizes it through a native enum/code converter, and writes the converted value to table-B row `i`. The consumer selects a row by explicit index and projects it to a separate 15-word local structure. Run-020 dynamically binds the SpatialCodingState construction/return, OAR nested-state field, frame consumer and table-B consumer on one Meter callback.

This closes the native producer/owner/consumer path for a native SpatialCoding metadata table. It does not name that table as product F06OF `control16`. The native table writer drops the source map key, and neither the native semantic key nor a typed source-key/target-slot/control-word join is observed. A 16-row count, 52-byte stride, local row index, or equal scalar cannot establish identity with F06OF.

## Drain separation

Run-019 callback 4247 is a 38-source/1536-sample input callback. Callback 4248 is a 0-source/0-sample source-empty candidate; OAR and metadata consumers still execute synchronously on that callback. It is not the F06OF model's 38x1536 positive-zero DRAIN. The three downstream zero-length DPLM submissions remain distinct. Native F06OF generation and a three-drain native match remain unobserved.

## Smallest remaining identity receipt

To close one field mapping, the evidence must bind, in the same run, the semantic task-map key/type, its ordinal and value, the conversion result, table-B destination row/field and consumer row index, plus an independent F06OF receipt for target slot `s`, control52 row/word, and control16 row `1+s` in model generation `g+1`. A scalar value alone is insufficient because the native table writer drops the key. The source package allows a maximum 4-byte metadata scalar if needed and no PCM/control-table payload reads; this package does not claim that capture happened.
