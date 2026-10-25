# BSI importer static identity and runtime destination join

Status: `STATIC_IMPORTER_REGION_IDENTITY_CLOSED; RUNTIME_DESTINATION_JOIN_UNBOUND`.

The hash-bound producer/capture package and the separately observed natural configuration-import package identify the same importer code region by SHA-256 `a6a85906698046134d08386b5e4532c797f8fded55f48b136a3c89d55da65ab1`. The reviewed washed importer contract describes a literal copy into a working header. The existing F12 wire-field contract describes direct U8 emission from working-header byte 119 to `complexity_index_type_a` at frame bit 105.

This closes the identity of a static code region and the operation class along the two documented paths. It does not identify a particular invocation, import destination, selected parent, or source generation.

The private count-capture record uses capture-local record token 6 under owner/config tokens 5/4. The natural configuration-import observation names `encoder_owner0` and outer generations 1 through 6. There is no shared destination/owner/call token joining that record-6 import to `encoder_owner0`'s working-header byte 119 in one generation. A matching static-region digest does not establish that runtime identity.

Keep the runtime importer-to-selected-header join, same-parent/cross-parent result, and source/frame/AU generation `UNBOUND`. Do not infer them from equal values, shared code identity, or call ordering. No additional algorithm rule or production-ready source mapping is established by this supplement.
