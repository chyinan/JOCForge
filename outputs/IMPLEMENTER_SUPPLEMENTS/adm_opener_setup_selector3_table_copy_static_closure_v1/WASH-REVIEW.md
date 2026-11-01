# Selector-3 opener-to-setup table copy boundary

This Wash addendum reclassifies the earlier opener-to-setup pointer question. Ghidra static flow shows the host receives the opener's master/header outputs through fields on one parent object. The opener writes its table pointer into the returned header. A later host stage clones that header, allocates a new table sized as `record_count * 6`, copies the table records, and passes the cloned header to the selector-3 setup callback. The setup consumer reads its table from `param4[2]`.

Therefore the static table-content lineage is bound, but the pointer is deliberately not an alias: the setup table is a new allocation populated by copying the opener records. This supersedes the prior interpretation that demanded pointer equality for opener `this+0x50` and setup `param4[2]`. Existing washed CHNA mapping defines row and payload meanings.

Runtime parent identity for a particular input and a job-generation token remain unobserved. The static path is selector-3-specific and does not claim a new DEE run, a universal route, or an audio algorithm rule.
