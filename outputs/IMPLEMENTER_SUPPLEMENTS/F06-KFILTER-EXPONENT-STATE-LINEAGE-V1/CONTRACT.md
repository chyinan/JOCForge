# F06 K-filter exponent state lineage

This supplement distinguishes an approved-model omission, a selected DPLM static member, and a separate sibling reconciliation candidate. It does not supply a selected-root cold/update rule.

The approved F06W model has six channels of [2][4] F32 K-filter history and no K-filter I32 exponent array. Its separate energy accumulator has an I32[6] exponent, cold 48, owned and updated by the energy-window recurrence; that field must not substitute for the K exponent.

Static instruction-byte-matched evidence reports that the DPLM 48 kHz reducer reads a distinct signed I32[6] K-associated exponent while normalizing filtered power; it separately reads/updates the energy accumulator exponent. The K row kernel uses F32 histories and does not write the K exponent in that callback chain. The DPLM constructor/first writer/cold/reset source and identity to owners.meter remain UNBOUND.

A sibling block-processing branch contains a static per-channel I32 reconciliation helper and max-absolute-input exponent source. It is not connected by the exported call graph to the selected DPLM owner; the helper formula below is a candidate only, not a selected DPLM rule. It also scans five words per stage while writing four, and the fifth word is not mapped to F06W.

No native execution, reset capture, or numeric replay was run. The only safe next step for selected-root closure is the DPLM K exponent constructor/first writer/reset path and its owner token. Keep the energy exponent separate and leave terminal meter digest/MeasurementReceipt blocked until the selected field owner is established.
