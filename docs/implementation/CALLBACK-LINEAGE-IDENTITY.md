# Callback lineage identity

This local join identity follows the Wash type-map erratum
`CALLBACK-LINEAGE-TYPEMAP-ERRATUM-V1` (manifest SHA-256
`e561b73cdab75187ab1df8ebcaa43d9a14f3dd232213809e19ea9f210a96adaf`).

`ProgrammeId` is a nonempty caller-supplied string. It stays stable across full
restart attempts and is not derived from input, configuration, or audio bytes.
`AttemptId` is U64 and advances only after the previous attempt is retired and
the complete cold reset succeeds. `FrontendCallbackIndex` remains U32.

`CallbackLineageIdentity` groups those three values for equality checks across
selected callback stages. It is an internal join value only: it is not a GTC
transaction ID, receipt capability, serialized format, or native ABI.
`CallbackTransactionId` remains the separate nonempty string in the approved
P1.3-C GTC schema. Its coordinator and receipt are still blocked on the
approved owner roots.

At a future GTC schema boundary, convert the U32 frontend callback index
explicitly to the schema's U64 `callback_index`. There is no coordinator or
GTC receipt serializer in the current implementation.
